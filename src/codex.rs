use crate::codex_options::CodexOptions;
use crate::error::CodexError;
use crate::exec::CodexExec;
use crate::thread::Thread;
use crate::thread_options::ThreadOptions;

#[derive(Clone, Debug)]
pub struct Codex {
    exec: CodexExec,
    options: CodexOptions,
}

impl Codex {
    pub fn new(options: CodexOptions) -> Result<Self, CodexError> {
        let exec = CodexExec::new(
            options.codex_path_override.clone(),
            options.env.clone(),
            options.config.clone(),
        )?;
        Ok(Self { exec, options })
    }

    pub fn start_thread(&self, options: ThreadOptions) -> Thread {
        Thread::new(self.exec.clone(), self.options.clone(), options, None)
    }

    pub fn resume_thread(&self, id: String, options: ThreadOptions) -> Thread {
        Thread::new(self.exec.clone(), self.options.clone(), options, Some(id))
    }
}
