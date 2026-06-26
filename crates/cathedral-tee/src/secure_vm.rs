use anyhow::Result;
use cathedral_lc3_vm::Lc3Vm;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SecureVmExecutor {
    vm: Arc<Mutex<Lc3Vm>>,
    max_instructions: u64,
    memory_limit: usize,
}

impl SecureVmExecutor {
    pub fn new(max_instructions: u64, memory_limit: usize) -> Self {
        Self { vm: Arc::new(Mutex::new(Lc3Vm::new())), max_instructions, memory_limit }
    }

    pub async fn execute_secure(&self, binary: &[u16], input: &str) -> Result<String> {
        let mut vm = self.vm.lock().await;
        *vm = Lc3Vm::new();
        vm.load_program(binary);
        vm.set_input(input);

        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio::task::spawn_blocking({
                let mut vm_clone = vm.clone();
                move || {
                    vm_clone.run()?;
                    Ok::<String, anyhow::Error>(vm_clone.get_output().to_string())
                }
            }),
        )
        .await??
    }
}
