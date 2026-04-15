Before running an evaluation script, be sure to
```
rm -rf ~/.malvin
make install
```

When you run a script, capture its stdout and stderr, like
```
./evaluations/eval_script.sh &> log_eval_script_1
```

After running, look in the directory where the script was working (which it will announce near the beginning of its output).
