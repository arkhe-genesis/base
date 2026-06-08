from cathedral.types import GGUFHeader, TensorInfo
import struct
import mmap
import os
import numpy as np
from collections import defaultdict
from pathlib import Path


class GGUFBridgeV3:
    def __init__(self, cache_size=10):
        self._mmap = None; self._file_path = None; self._tensor_data_offset = 0
        self.header = None; self.metadata = {}; self.tensors = []
        self.file_size = 0; self._cache = {}; self._cache_size = cache_size
        self._access_count = defaultdict(int)

    def _read_string(self, f):
        length = struct.unpack("<Q", f.read(8))[0]
        return f.read(length).decode("utf-8")

    def _read_value(self, f, type_code):
        _r = {
            0: lambda: struct.unpack("<B", f.read(1))[0], 1: lambda: struct.unpack("<b", f.read(1))[0],
            2: lambda: struct.unpack("<H", f.read(2))[0], 3: lambda: struct.unpack("<h", f.read(2))[0],
            4: lambda: struct.unpack("<I", f.read(4))[0], 5: lambda: struct.unpack("<i", f.read(4))[0],
            6: lambda: struct.unpack("<f", f.read(4))[0], 7: lambda: struct.unpack("<B", f.read(1))[0] != 0,
            8: lambda: self._read_string(f), 10: lambda: struct.unpack("<Q", f.read(8))[0],
            11: lambda: struct.unpack("<q", f.read(8))[0], 12: lambda: struct.unpack("<d", f.read(8))[0],
        }
        if type_code == 9:
            et = struct.unpack("<I", f.read(4))[0]
            n = struct.unpack("<Q", f.read(8))[0]
            return [self._read_value(f, et) for _ in range(n)]
        return _r.get(type_code, lambda: None)()

    def open(self, file_path):
        self.close()
        if not Path(file_path).exists(): return False
        self.file_size = Path(file_path).stat().st_size
        self._file_path = file_path
        fd = os.open(file_path, os.O_RDONLY)
        self._mmap = mmap.mmap(fd, self.file_size, access=mmap.ACCESS_READ)
        os.close(fd)
        self.header = GGUFHeader(
            struct.unpack("<I", self._mmap[0:4])[0],
            struct.unpack("<I", self._mmap[4:8])[0],
            struct.unpack("<Q", self._mmap[8:16])[0],
            struct.unpack("<Q", self._mmap[16:24])[0],
        )
        if not self.header.valid: self.close(); return False
        from io import BytesIO
        f = BytesIO(self._mmap); f.seek(24)
        self.metadata = {}
        for _ in range(self.header.metadata_kv_count):
            key = self._read_string(f); tc = struct.unpack("<I", f.read(4))[0]
            self.metadata[key] = self._read_value(f, tc)
        self.tensors = []
        for _ in range(self.header.tensor_count):
            name = self._read_string(f); nd = struct.unpack("<I", f.read(4))[0]
            dims = [struct.unpack("<Q", f.read(8))[0] for _ in range(nd)]
            tc = struct.unpack("<I", f.read(4))[0]; off = struct.unpack("<Q", f.read(8))[0]
            self.tensors.append(TensorInfo(name, nd, dims, tc, off))
        self._tensor_data_offset = f.tell()
        pad = (32 - self._tensor_data_offset % 32) % 32
        self._tensor_data_offset += pad
        return True

    def read_tensor_data(self, tensor_name, slice_indices=None):
        if tensor_name in self._cache and slice_indices is None:
            self._access_count[tensor_name] += 1; return self._cache[tensor_name]
        t = next((t for t in self.tensors if t.name == tensor_name), None)
        if not t or not self._mmap: return None
        off = self._tensor_data_offset + t.offset
        if t.type_code == 0:
            data = np.frombuffer(self._mmap, dtype=np.float32, count=t.num_elements, offset=off).reshape(t.dims)
        elif t.type_code == 1:
            data = np.frombuffer(self._mmap, dtype=np.float16, count=t.num_elements, offset=off).astype(np.float32).reshape(t.dims)
        else:
            data = np.frombuffer(self._mmap, dtype=np.uint8, count=t.size_bytes, offset=off)
        if slice_indices: data = data[slice_indices]
        if slice_indices is None:
            if len(self._cache) >= self._cache_size:
                min_key = min(self._cache.keys(), key=lambda k: self._access_count.get(k, 0))
                del self._cache[min_key]; del self._access_count[min_key]
            self._cache[tensor_name] = data; self._access_count[tensor_name] = 1
        return data

    def get_architecture(self): return self.metadata.get("general.architecture", "unknown")
    def get_context_length(self): return self.metadata.get(f"{self.get_architecture()}.context_length", 0)
    def get_embedding_length(self): return self.metadata.get(f"{self.get_architecture()}.embedding_length", 0)
    def get_block_count(self): return self.metadata.get(f"{self.get_architecture()}.block_count", 0)
    def get_head_count(self): return self.metadata.get(f"{self.get_architecture()}.attention.head_count", 0)

    def close(self):
        if self._mmap: self._mmap.close(); self._mmap = None
        self._cache.clear(); self._access_count.clear()
    def __enter__(self): return self
    def __exit__(self, *a): self.close()

    def get_telemetry(self):
        return {"module": "GGUFBridgeV3", "version": "3.0.0", "substrate": "1094.1",
                "seal": "GGUF-BRIDGE-1094.1-v3.0.0-2026-06-07",
                "file": self._file_path, "file_size": self.file_size,
                "mmap_active": self._mmap is not None, "cache_entries": len(self._cache),
                "tensors_loaded": len(self.tensors), "metadata_keys": len(self.metadata)}
