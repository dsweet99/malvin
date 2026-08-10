/**
 * Exit when the parent process disappears.
 *
 * Must load before heavy imports: otherwise SIGKILL of the parent during module
 * init can orphan us under a new PPID before the watch is armed, and a duplicated
 * stdin write-end prevents EOF-based shutdown.
 */
const parentPid = process.ppid;
export function installParentDeathWatch(exitFn = (code) => process.exit(code), intervalMs = 100) {
    const timer = setInterval(() => {
        if (process.ppid !== parentPid) {
            exitFn(0);
        }
    }, intervalMs);
    timer.unref?.();
    return timer;
}
// Arm immediately at module evaluation (side effect on import).
installParentDeathWatch();
