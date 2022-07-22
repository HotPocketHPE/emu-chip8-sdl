pub enum RunMode {
    Normal,
    DebugStep,
    DebugRunning,
    DebugCond,
    DebugCondStep,
}

impl RunMode {
    pub fn pause(&mut self) -> Result<RunMode, String>{
        match self {
            RunMode::DebugRunning => Ok(RunMode::DebugStep),
            RunMode::DebugCond => Ok(RunMode::DebugCondStep),
            _ => return Err("Cannot pause this run mode".into()),
        }
    }

    pub fn resume(&mut self) -> Result<RunMode, String>{
        match self {
            RunMode::DebugStep => Ok(RunMode::DebugRunning),
            RunMode::DebugCondStep => Ok(RunMode::DebugCond),
            _ => return Err("Cannot resume this run mode".into()),
        }
    }
}