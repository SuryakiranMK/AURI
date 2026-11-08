# PERFORMANCE TARGETS & BENCHMARK SUITE: PROJECT AURI

---

## 1. Measurable Resource Budgets

Resource targets are hard engineering boundaries, not vague guidelines. Every component introduced into AURI must prove it fits within these budgets on the reference development system (Intel Core i7-11800H, NVIDIA RTX 3050 Ti, 16 GB RAM, Windows 11).

| Operating State | Working Set (RAM) | Private Bytes | Idle CPU | Active Playback CPU | GPU Load | Process Count |
|---|---|---|---|---|---|---|
| **App Idle (Normal Mode)** | **< 60 MB** | < 45 MB | < 0.1% | N/A | 0.0% | **1** |
| **Normal Mode (Playing)** | **< 75 MB** | < 55 MB | N/A | **< 0.8%** | < 0.5% | **1** |
| **Mini Mode (Playing)** | **< 50 MB** | < 38 MB | N/A | **< 0.6%** | < 0.2% | **1** |
| **Gaming Mode (Playing)** | **< 35 MB** | < 25 MB | N/A | **< 0.4%** | **0.0%** | **1** |
| **Demanding Game Active** | **< 35 MB** | < 25 MB | N/A | **< 0.4%** | **0.0%** | **1** |

---

## 2. Competitive Benchmark Comparison

Baseline resource comparison between leading music solutions on Windows 11:

```
+-----------------------------------------------------------------------------------------+
| Player                   | Working Set (RAM) | Background CPU | GPU Engine | Processes  |
+--------------------------+-------------------+----------------+------------+------------+
| YouTube Music (Chrome)   | 450 MB - 850 MB   | 1.5% - 4.5%    | Active     | 8 - 14     |
| Spotify Desktop          | 320 MB - 600 MB   | 1.0% - 3.0%    | Active     | 5 - 8      |
| Baseline egui Music Player | 95 MB - 130 MB    | 0.5% - 1.2%    | Low/Active | 1          |
| foobar2000 (Win32/C++)   | 18 MB - 30 MB     | 0.1% - 0.5%    | 0.0%       | 1          |
+--------------------------+-------------------+----------------+------------+------------+
| AURI (Normal Mode)       | < 75 MB           | < 0.8%         | < 0.5%     | 1          |
| AURI (Gaming Mode)       | < 35 MB           | < 0.4%         | 0.0%       | 1          |
+-----------------------------------------------------------------------------------------+
```

---

## 3. Playback Latency & Responsiveness Budgets

- **Cold Startup Time (to UI interactive)**: < 400 milliseconds.
- **Track Start Latency (Local file)**: < 15 milliseconds.
- **Track Seek Latency (Local file)**: < 20 milliseconds.
- **Track Start Latency (Streaming buffer)**: < 250 milliseconds (dependent on network ping).
- **Buffer Underruns / Audio Dropouts**: Strictly **0** during 100% CPU stress test (using real-time MMCSS audio priority).
- **Gaming Mode Transition Time**: < 50 milliseconds to hide UI, suspend swapchain, and trigger working set purge.

---

## 4. Measurement Methodology & Tooling

To ensure scientific rigor, performance must be verified using reproducible scripts and native Windows performance APIs:

1. **Memory Tracking**:
   - `GetProcessMemoryInfo` (retrieving `WorkingSetSize`, `PeakWorkingSetSize`, `PrivateUsage`).
   - Monitored via Windows Performance Monitor (`perfmon`) and automated PowerShell audit scripts.
2. **CPU & Thread Tracking**:
   - `GetProcessTimes` (KernelTime + UserTime calculation over delta intervals).
   - Thread count monitored via Windows `Toolhelp32Snapshot`.
3. **GPU Utilization**:
   - Monitored via Windows D3D Performance Counters (`\GPU Engine(*)\Utilization Percentage`) to ensure 0.0% in Gaming Mode.
4. **Automated Benchmark Script**:
   - A dedicated PowerShell/Python profiling script (`scripts/benchmark.ps1`) will be integrated into the repository to run automated stress tests across all three operating modes.

