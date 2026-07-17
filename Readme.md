# rtime ⏱️

A fast, lightweight, and highly accurate clone of the Unix/Linux `time` command, built specifically for Windows using Rust and raw WinAPI. 

`rtime` measures the execution time of a given command, reporting the **real** (wall-clock), **user** (CPU time in user space), and **sys** (CPU time in kernel space) metrics.

## ✨ Features

* 🚀 **Extremely Lightweight:** Compiled as a tiny single binary (can be optimized to under 500KB). No heavy dependencies.
* 🎯 **True Process Tree Tracking:** Unlike naive implementations that use `GetProcessTimes` on a single process, `rtime` utilizes **Windows Job Objects**. This ensures that all spawned child processes (e.g., calling `npm` which spawns `node.exe`) are accurately accounted for in the total CPU time.
* ⚡ **Dual-Engine Precision:** Includes a fallback mechanism to capture CPU times for extremely short-lived processes that might execute faster than the Job Object's accounting tick rate (15.6ms).
* 🛠 **Smart Built-in Command Support:** Automatically detects Windows built-in commands (like `dir`, `echo`, or `cls`) and safely wraps them in `cmd.exe /C` under the hood.

## 📦 Installation

Ensure you have [Rust and Cargo](https://rustup.rs/) installed on your Windows machine.

1. Clone this repository or download the source code:
   ```bash
   git clone https://github.com/py7hon/rtime.git
   cd rtime
   ```
2. Build the executable in release mode:
   ```bash
   cargo build --release
   ```
3. The compiled binary will be located at `target\release\rtime.exe`. 
4. **(Recommended)** Add the `.exe` to your System `PATH` so you can call `rtime` from any terminal window.

### Size Optimization
The `Cargo.toml` is already pre-configured for size optimization (`opt-level = "z"`, `lto = true`, etc.). For even smaller binaries, you can pack the resulting executable using [UPX](https://upx.github.io/):
```bash
upx --best --lzma target/release/rtime.exe
```

## 🚀 Usage

Simply prefix any command you want to measure with `rtime`:

```bash
rtime <command> [arguments...]
```

### Examples

**1. Standard executables:**
```bash
rtime ping 127.0.0.1 -n 3
```

**2. Built-in shell commands:**
```bash
rtime dir
```

**3. CPU-intensive tasks (Multi-threading example):**
```bash
rtime powershell -Command "1..1000000 | Measure-Object"
```

### Understanding the Output

When the process finishes, `rtime` prints three metrics to `stderr` (so it won't interfere with your command's standard output piping):

```text
real    1.888s
user    3.422s
sys     0.688s
```

*   **real**: Wall-clock time. The actual time elapsed from start to finish.
*   **user**: Total amount of CPU time spent in user mode. *(Note: If your process uses multiple CPU cores, this value can be higher than the `real` time).*
*   **sys**: Total amount of CPU time spent in the kernel (operating system calls, I/O, etc.).

## 🛠 How it Works (Under the Hood)

Windows does not have a direct equivalent to Unix's `wait4` / `rusage` that works seamlessly across process trees. 

To solve this, `rtime` creates a nameless **Job Object** via WinAPI (`CreateJobObjectW`). The target process is immediately assigned to this job (`AssignProcessToJobObject`). Any child processes spawned by the target will automatically join this Job Object. Once the target process exits, `rtime` queries the Job Object's accounting information (`JOBOBJECT_BASIC_ACCOUNTING_INFORMATION`) to get the exact `TotalUserTime` and `TotalKernelTime` for the entire process tree.

## 📄 License

This project is licensed under the MIT License.