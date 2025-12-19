from typing import Optional

class SandboxConfig:
    log_directory: Optional[str]
    tmp_directory: Optional[str]
    def __init__(self, log_directory: Optional[str] = None, tmp_directory: Optional[str] = None) -> None: ...

class WorkloadResult:
    success: bool
    error: Optional[str]

class NanvixSandbox:
    def __init__(self, config: Optional[SandboxConfig] = None) -> None: ...
    async def run(self, workload_path: str) -> WorkloadResult: ...
    async def clear_cache(self) -> None: ...

__all__ = ["NanvixSandbox", "SandboxConfig", "WorkloadResult"]
