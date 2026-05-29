// arkhe.ko — Linux Kernel Module (LKM)
// Substrato 274: Intercepta syscalls e ancora eventos na TemporalChain.
// Compilar com: make (Kbuild) + rustc --target x86_64-unknown-linux-musl

#![no_std]
#![feature(allocator_api)]

use kernel::prelude::*;
use kernel::netlink::{NetlinkSocket, NetlinkMessage};
use kernel::random::getrandom;
use kernel::crypto::{sha3_256, ed25519_sign};
use kernel::task::Task;

module! {
    type: ArkheModule,
    name: "arkhe",
    author: "ARKHE-OS Architect (ORCID 0009-0005-2697-4668)",
    description: "Kernel module for TemporalChain anchoring",
    license: "GPL v2",
}

struct ArkheModule {
    _netlink: NetlinkSocket,
    node_key: [u8; 64], // Ed25519 private key
}

impl KernelModule for ArkheModule {
    fn init(module: &'static ThisModule) -> Result<Self> {
        let netlink = NetlinkSocket::new(module, 31 /* ARKHE_FAMILY */)?;
        let node_key = load_key_from_tpm()?;

        // Registra hooks de syscall
        register_syscall_hook(SyscallOp::Open, sys_open_hook)?;
        register_syscall_hook(SyscallOp::Write, sys_write_hook)?;
        register_syscall_hook(SyscallOp::Execve, sys_execve_hook)?;

        Ok(ArkheModule { _netlink: netlink, node_key })
    }
}

// Hook para sys_open: captura abertura de arquivos
fn sys_open_hook(filename: &[u8], flags: u32, mode: u16) -> i64 {
    let event = construct_event("OPEN", filename, flags, mode);
    queue_event(&event);
    0 // prossegue execução normal
}

// Hook para sys_write: captura escrita em arquivos
fn sys_write_hook(fd: u32, buf: &[u8], count: usize) -> i64 {
    let event = construct_event_with_payload("WRITE", fd, buf, count);
    queue_event(&event);
    0
}

// Hook para sys_execve: captura execução de programas
fn sys_execve_hook(filename: &[u8], argv: &[&[u8]]) -> i64 {
    let event = construct_event("EXEC", filename, 0, 0);
    queue_event(&event);
    0
}

fn construct_event(op: &str, filename: &[u8], _flags: u32, _mode: u16) -> KernelEvent {
    let mut hasher = Sha3_256::new();
    hasher.update(op.as_bytes());
    hasher.update(filename);
    hasher.update(&current_task().pid.to_le_bytes());
    hasher.update(&get_time_ns().to_le_bytes());
    let hash = hasher.finalize();

    KernelEvent {
        op: op.to_string(),
        path: String::from_utf8_lossy(filename).to_string(),
        pid: current_task().pid,
        timestamp_ns: get_time_ns(),
        hash,
        signature: vec![], // será assinado na fila
    }
}
