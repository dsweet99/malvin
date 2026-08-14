

const parentPid = process.ppid;

export function installParentDeathWatch(
  exitFn: (code: number) => void = (code) => process.exit(code),
  intervalMs = 100,
): NodeJS.Timeout {
  const timer = setInterval(() => {
    if (process.ppid !== parentPid) {
      exitFn(0);
    }
  }, intervalMs);
  timer.unref?.();
  return timer;
}

installParentDeathWatch();
