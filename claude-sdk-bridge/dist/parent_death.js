const parentPid = process.ppid;
setInterval(() => {
    if (process.ppid === 1 || process.ppid !== parentPid)
        process.exit(0);
}, 1_000).unref();
export {};
