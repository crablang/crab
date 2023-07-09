#!/usr/bin/env python3
# ignore-tidy-linelength

# Compatible with Python 3.6+

import contextlib
import getpass
import glob
import json
import logging
import os
import pprint
import shutil
import subprocess
import sys
import time
import traceback
import urllib.request
from io import StringIO
from pathlib import Path
from typing import Callable, ContextManager, Dict, Iterable, Iterator, List, Optional, \
    Tuple, Union

PGO_HOST = os.environ["PGO_HOST"]

LOGGER = logging.getLogger("stage-build")

LLVM_PGO_CRATES = [
    "syn-1.0.89",
    "cargo-0.60.0",
    "serde-1.0.136",
    "ripgrep-13.0.0",
    "regex-1.5.5",
    "clap-3.1.6",
    "hyper-0.14.18"
]

RUSTC_PGO_CRATES = [
    "externs",
    "ctfe-stress-5",
    "cargo-0.60.0",
    "token-stream-stress",
    "match-stress",
    "tuple-stress",
    "diesel-1.4.8",
    "bitmaps-3.1.0"
]

LLVM_BOLT_CRATES = LLVM_PGO_CRATES


def is_try_build() -> bool:
    return os.environ.get("DIST_TRY_BUILD", "0") != "0"


class Pipeline:
    # Paths
    def checkout_path(self) -> Path:
        """
        The root checkout, where the source is located.
        """
        raise NotImplementedError

    def downloaded_llvm_dir(self) -> Path:
        """
        Directory where the host LLVM is located.
        """
        raise NotImplementedError

    def build_root(self) -> Path:
        """
        The main directory where the build occurs.
        """
        raise NotImplementedError

    def build_artifacts(self) -> Path:
        return self.build_root() / "build" / PGO_HOST

    def rustc_stage_0(self) -> Path:
        return self.build_artifacts() / "stage0" / "bin" / "rustc"

    def cargo_stage_0(self) -> Path:
        return self.build_artifacts() / "stage0" / "bin" / "cargo"

    def rustc_stage_2(self) -> Path:
        return self.build_artifacts() / "stage2" / "bin" / "rustc"

    def opt_artifacts(self) -> Path:
        raise NotImplementedError

    def llvm_profile_dir_root(self) -> Path:
        return self.opt_artifacts() / "llvm-pgo"

    def llvm_profile_merged_file(self) -> Path:
        return self.opt_artifacts() / "llvm-pgo.profdata"

    def rustc_perf_dir(self) -> Path:
        return self.opt_artifacts() / "rustc-perf"

    def build_rustc_perf(self):
        raise NotImplementedError()

    def rustc_profile_dir_root(self) -> Path:
        return self.opt_artifacts() / "rustc-pgo"

    def rustc_profile_merged_file(self) -> Path:
        return self.opt_artifacts() / "rustc-pgo.profdata"

    def rustc_profile_template_path(self) -> Path:
        """
        The profile data is written into a single filepath that is being repeatedly merged when each
        rustc invocation ends. Empirically, this can result in some profiling data being lost. That's
        why we override the profile path to include the PID. This will produce many more profiling
        files, but the resulting profile will produce a slightly faster rustc binary.
        """
        return self.rustc_profile_dir_root() / "default_%m_%p.profraw"

    def supports_bolt(self) -> bool:
        raise NotImplementedError

    def llvm_bolt_profile_merged_file(self) -> Path:
        return self.opt_artifacts() / "bolt.profdata"

    def metrics_path(self) -> Path:
        return self.build_root() / "build" / "metrics.json"

    def executable_extension(self) -> str:
        raise NotImplementedError

    def skipped_tests(self) -> Iterable[str]:
        return ()


class LinuxPipeline(Pipeline):
    def checkout_path(self) -> Path:
        return Path("/checkout")

    def downloaded_llvm_dir(self) -> Path:
        return Path("/rustroot")

    def build_root(self) -> Path:
        return self.checkout_path() / "obj"

    def opt_artifacts(self) -> Path:
        return Path("/tmp/tmp-multistage/opt-artifacts")

    def build_rustc_perf(self):
        # /tmp/rustc-perf comes from the Dockerfile
        shutil.copytree("/tmp/rustc-perf", self.rustc_perf_dir())
        cmd(["chown", "-R", f"{getpass.getuser()}:", self.rustc_perf_dir()])

        with change_cwd(self.rustc_perf_dir()):
            cmd([self.cargo_stage_0(), "build", "-p", "collector"], env=dict(
                RUSTC=str(self.rustc_stage_0()),
                RUSTC_BOOTSTRAP="1"
            ))

    def supports_bolt(self) -> bool:
        return True

    def executable_extension(self) -> str:
        return ""

    def skipped_tests(self) -> Iterable[str]:
        # This test fails because of linker errors, as of June 2023.
        yield "tests/ui/process/nofile-limit.rs"


class WindowsPipeline(Pipeline):
    def __init__(self):
        self.checkout_dir = Path(os.getcwd())

    def checkout_path(self) -> Path:
        return self.checkout_dir

    def downloaded_llvm_dir(self) -> Path:
        return self.checkout_path() / "citools" / "clang-rust"

    def build_root(self) -> Path:
        return self.checkout_path()

    def opt_artifacts(self) -> Path:
        return self.checkout_path() / "opt-artifacts"

    def rustc_stage_0(self) -> Path:
        return super().rustc_stage_0().with_suffix(".exe")

    def cargo_stage_0(self) -> Path:
        return super().cargo_stage_0().with_suffix(".exe")

    def rustc_stage_2(self) -> Path:
        return super().rustc_stage_2().with_suffix(".exe")

    def build_rustc_perf(self):
        # rustc-perf version from 2023-03-15
        perf_commit = "8b2ac3042e1ff2c0074455a0a3618adef97156b1"
        rustc_perf_zip_path = self.opt_artifacts() / "perf.zip"

        def download_rustc_perf():
            download_file(
                f"https://github.com/rust-lang/rustc-perf/archive/{perf_commit}.zip",
                rustc_perf_zip_path
            )
            with change_cwd(self.opt_artifacts()):
                unpack_archive(rustc_perf_zip_path)
                move_path(Path(f"rustc-perf-{perf_commit}"), self.rustc_perf_dir())
                delete_file(rustc_perf_zip_path)

        retry_action(download_rustc_perf, "Download rustc-perf")

        with change_cwd(self.rustc_perf_dir()):
            cmd([self.cargo_stage_0(), "build", "-p", "collector"], env=dict(
                RUSTC=str(self.rustc_stage_0()),
                RUSTC_BOOTSTRAP="1"
            ))

    def rustc_profile_template_path(self) -> Path:
        """
        On Windows, we don't have enough space to use separate files for each rustc invocation.
        Therefore, we use a single file for the generated profiles.
        """
        return self.rustc_profile_dir_root() / "default_%m.profraw"

    def supports_bolt(self) -> bool:
        return False

    def executable_extension(self) -> str:
        return ".exe"

    def skipped_tests(self) -> Iterable[str]:
        # This test fails as of June 2023
        yield "tests\\codegen\\vec-shrink-panik.rs"


def get_timestamp() -> float:
    return time.time()


Duration = float


def iterate_timers(timer: "Timer", name: str, level: int = 0) -> Iterator[
    Tuple[int, str, Duration]]:
    """
    Hierarchically iterate the children of a timer, in a depth-first order.
    """
    yield (level, name, timer.total_duration())
    for (child_name, child_timer) in timer.children:
        yield from iterate_timers(child_timer, child_name, level=level + 1)


class Timer:
    def __init__(self, parent_names: Tuple[str, ...] = ()):
        self.children: List[Tuple[str, Timer]] = []
        self.section_active = False
        self.parent_names = parent_names
        self.duration_excluding_children: Duration = 0

    @contextlib.contextmanager
    def section(self, name: str) -> ContextManager["Timer"]:
        assert not self.section_active
        self.section_active = True

        start = get_timestamp()
        exc = None

        child_timer = Timer(parent_names=self.parent_names + (name,))
        full_name = " > ".join(child_timer.parent_names)
        try:
            LOGGER.info(f"Section `{full_name}` starts")
            yield child_timer
        except BaseException as exception:
            exc = exception
            raise
        finally:
            end = get_timestamp()
            duration = end - start

            child_timer.duration_excluding_children = duration - child_timer.total_duration()
            self.add_child(name, child_timer)
            if exc is None:
                LOGGER.info(f"Section `{full_name}` ended: OK ({duration:.2f}s)")
            else:
                LOGGER.info(f"Section `{full_name}` ended: FAIL ({duration:.2f}s)")
            self.section_active = False

    def total_duration(self) -> Duration:
        return self.duration_excluding_children + sum(
            c.total_duration() for (_, c) in self.children)

    def has_children(self) -> bool:
        return len(self.children) > 0

    def print_stats(self):
        rows = []
        for (child_name, child_timer) in self.children:
            for (level, name, duration) in iterate_timers(child_timer, child_name, level=0):
                label = f"{'  ' * level}{name}:"
                rows.append((label, duration))

        # Empty row
        rows.append(("", ""))

        total_duration_label = "Total duration:"
        total_duration = self.total_duration()
        rows.append((total_duration_label, humantime(total_duration)))

        space_after_label = 2
        max_label_length = max(16, max(len(label) for (label, _) in rows)) + space_after_label

        table_width = max_label_length + 23
        divider = "-" * table_width

        with StringIO() as output:
            print(divider, file=output)
            for (label, duration) in rows:
                if isinstance(duration, Duration):
                    pct = (duration / total_duration) * 100
                    value = f"{duration:>12.2f}s ({pct:>5.2f}%)"
                else:
                    value = f"{duration:>{len(total_duration_label) + 7}}"
                print(f"{label:<{max_label_length}} {value}", file=output)
            print(divider, file=output, end="")
            LOGGER.info(f"Timer results\n{output.getvalue()}")

    def add_child(self, name: str, timer: "Timer"):
        self.children.append((name, timer))

    def add_duration(self, name: str, duration: Duration):
        timer = Timer(parent_names=self.parent_names + (name,))
        timer.duration_excluding_children = duration
        self.add_child(name, timer)


class BuildStep:
    def __init__(self, type: str, children: List["BuildStep"], duration: float):
        self.type = type
        self.children = children
        self.duration = duration

    def find_all_by_type(self, type: str) -> Iterator["BuildStep"]:
        if type == self.type:
            yield self
        for child in self.children:
            yield from child.find_all_by_type(type)

    def __repr__(self):
        return f"BuildStep(type={self.type}, duration={self.duration}, children={len(self.children)})"


def load_last_metrics(path: Path) -> BuildStep:
    """
    Loads the metrics of the most recent bootstrap execution from a metrics.json file.
    """
    with open(path, "r") as f:
        metrics = json.load(f)
    invocation = metrics["invocations"][-1]

    def parse(entry) -> Optional[BuildStep]:
        if "kind" not in entry or entry["kind"] != "rustbuild_step":
            return None
        type = entry.get("type", "")
        duration = entry.get("duration_excluding_children_sec", 0)
        children = []

        for child in entry.get("children", ()):
            step = parse(child)
            if step is not None:
                children.append(step)
                duration += step.duration
        return BuildStep(type=type, children=children, duration=duration)

    children = [parse(child) for child in invocation.get("children", ())]
    return BuildStep(
        type="root",
        children=children,
        duration=invocation.get("duration_including_children_sec", 0)
    )


@contextlib.contextmanager
def change_cwd(dir: Path):
    """
    Temporarily change working directory to `dir`.
    """
    cwd = os.getcwd()
    LOGGER.debug(f"Changing working dir from `{cwd}` to `{dir}`")
    os.chdir(dir)
    try:
        yield
    finally:
        LOGGER.debug(f"Reverting working dir to `{cwd}`")
        os.chdir(cwd)


def humantime(time_s: float) -> str:
    hours = time_s // 3600
    time_s = time_s % 3600
    minutes = time_s // 60
    seconds = time_s % 60

    result = ""
    if hours > 0:
        result += f"{int(hours)}h "
    if minutes > 0:
        result += f"{int(minutes)}m "
    result += f"{round(seconds)}s"
    return result


def move_path(src: Path, dst: Path):
    LOGGER.info(f"Moving `{src}` to `{dst}`")
    shutil.move(src, dst)


def delete_file(path: Path):
    LOGGER.info(f"Deleting file `{path}`")
    os.unlink(path)


def delete_directory(path: Path):
    LOGGER.info(f"Deleting directory `{path}`")
    shutil.rmtree(path)


def unpack_archive(archive: Path, target_dir: Optional[Path] = None):
    LOGGER.info(f"Unpacking archive `{archive}`")
    shutil.unpack_archive(str(archive), extract_dir=str(target_dir) if target_dir is not None else None)


def download_file(src: str, target: Path):
    LOGGER.info(f"Downloading `{src}` into `{target}`")
    urllib.request.urlretrieve(src, str(target))


def retry_action(action, name: str, max_fails: int = 5):
    LOGGER.info(f"Attempting to perform action `{name}` with retry")
    for iteration in range(max_fails):
        LOGGER.info(f"Attempt {iteration + 1}/{max_fails}")
        try:
            action()
            return
        except BaseException: # also catch ctrl+c/sysexit
            LOGGER.error(f"Action `{name}` has failed\n{traceback.format_exc()}")

    raise Exception(f"Action `{name}` has failed after {max_fails} attempts")


def cmd(
        args: List[Union[str, Path]],
        env: Optional[Dict[str, str]] = None,
        output_path: Optional[Path] = None
):
    args = [str(arg) for arg in args]

    environment = os.environ.copy()

    cmd_str = ""
    if env is not None:
        environment.update(env)
        cmd_str += " ".join(f"{k}={v}" for (k, v) in (env or {}).items())
        cmd_str += " "
    cmd_str += " ".join(args)
    if output_path is not None:
        cmd_str += f" > {output_path}"
    LOGGER.info(f"Executing `{cmd_str}`")

    if output_path is not None:
        with open(output_path, "w") as f:
            return subprocess.run(
                args,
                env=environment,
                check=True,
                stdout=f
            )
    return subprocess.run(args, env=environment, check=True)


class BenchmarkRunner:
    def run_rustc(self, pipeline: Pipeline):
        raise NotImplementedError

    def run_llvm(self, pipeline: Pipeline):
        raise NotImplementedError

    def run_bolt(self, pipeline: Pipeline):
        raise NotImplementedError


class DefaultBenchmarkRunner(BenchmarkRunner):
    def run_rustc(self, pipeline: Pipeline):
        # Here we're profiling the `rustc` frontend, so we also include `Check`.
        # The benchmark set includes various stress tests that put the frontend under pressure.
        run_compiler_benchmarks(
            pipeline,
            profiles=["Check", "Debug", "Opt"],
            scenarios=["All"],
            crates=RUSTC_PGO_CRATES,
            env=dict(
                LLVM_PROFILE_FILE=str(pipeline.rustc_profile_template_path())
            )
        )

    def run_llvm(self, pipeline: Pipeline):
        run_compiler_benchmarks(
            pipeline,
            profiles=["Debug", "Opt"],
            scenarios=["Full"],
            crates=LLVM_PGO_CRATES
        )

    def run_bolt(self, pipeline: Pipeline):
        run_compiler_benchmarks(
            pipeline,
            profiles=["Check", "Debug", "Opt"],
            scenarios=["Full"],
            crates=LLVM_BOLT_CRATES
        )


def run_compiler_benchmarks(
        pipeline: Pipeline,
        profiles: List[str],
        scenarios: List[str],
        crates: List[str],
        env: Optional[Dict[str, str]] = None
):
    env = env if env is not None else {}

    # Compile libcore, both in opt-level=0 and opt-level=3
    with change_cwd(pipeline.build_root()):
        cmd([
            pipeline.rustc_stage_2(),
            "--edition", "2021",
            "--crate-type", "lib",
            str(pipeline.checkout_path() / "library/core/src/lib.rs"),
            "--out-dir", pipeline.opt_artifacts()
        ], env=dict(RUSTC_BOOTSTRAP="1", **env))

        cmd([
            pipeline.rustc_stage_2(),
            "--edition", "2021",
            "--crate-type", "lib",
            "-Copt-level=3",
            str(pipeline.checkout_path() / "library/core/src/lib.rs"),
            "--out-dir", pipeline.opt_artifacts()
        ], env=dict(RUSTC_BOOTSTRAP="1", **env))

    # Run rustc-perf benchmarks
    # Benchmark using profile_local with eprintln, which essentially just means
    # don't actually benchmark -- just make sure we run rustc a bunch of times.
    with change_cwd(pipeline.rustc_perf_dir()):
        cmd([
            pipeline.cargo_stage_0(),
            "run",
            "-p", "collector", "--bin", "collector", "--",
            "profile_local", "eprintln",
            pipeline.rustc_stage_2(),
            "--id", "Test",
            "--cargo", pipeline.cargo_stage_0(),
            "--profiles", ",".join(profiles),
            "--scenarios", ",".join(scenarios),
            "--include", ",".join(crates)
        ], env=dict(
            RUST_LOG="collector=debug",
            RUSTC=str(pipeline.rustc_stage_0()),
            RUSTC_BOOTSTRAP="1",
            **env
        ))


# https://stackoverflow.com/a/31631711/1107768
def format_bytes(size: int) -> str:
    """Return the given bytes as a human friendly KiB, MiB or GiB string."""
    KB = 1024
    MB = KB ** 2  # 1,048,576
    GB = KB ** 3  # 1,073,741,824
    TB = KB ** 4  # 1,099,511,627,776

    if size < KB:
        return f"{size} B"
    elif KB <= size < MB:
        return f"{size / KB:.2f} KiB"
    elif MB <= size < GB:
        return f"{size / MB:.2f} MiB"
    elif GB <= size < TB:
        return f"{size / GB:.2f} GiB"
    else:
        return str(size)


# https://stackoverflow.com/a/63307131/1107768
def count_files(path: Path) -> int:
    return sum(1 for p in path.rglob("*") if p.is_file())


def count_files_with_prefix(path: Path) -> int:
    return sum(1 for p in glob.glob(f"{path}*") if Path(p).is_file())


# https://stackoverflow.com/a/55659577/1107768
def get_path_size(path: Path) -> int:
    if path.is_dir():
        return sum(p.stat().st_size for p in path.rglob("*"))
    return path.stat().st_size


def get_path_prefix_size(path: Path) -> int:
    """
    Get size of all files beginning with the prefix `path`.
    Alternative to shell `du -sh <path>*`.
    """
    return sum(Path(p).stat().st_size for p in glob.glob(f"{path}*"))


def get_files(directory: Path, filter: Optional[Callable[[Path], bool]] = None) -> Iterable[Path]:
    for file in os.listdir(directory):
        path = directory / file
        if filter is None or filter(path):
            yield path


def bootstrap_build(
        pipeline: Pipeline,
        args: List[str],
        env: Optional[Dict[str, str]] = None,
        targets: Iterable[str] = ("library/std", )
):
    if env is None:
        env = {}
    else:
        env = dict(env)
    env["RUST_BACKTRACE"] = "1"
    arguments = [
                    sys.executable,
                    pipeline.checkout_path() / "x.py",
                    "build",
                    "--target", PGO_HOST,
                    "--host", PGO_HOST,
                    "--stage", "2",
                    ] + list(targets) + args
    cmd(arguments, env=env)


def create_pipeline() -> Pipeline:
    if sys.platform == "linux":
        return LinuxPipeline()
    elif sys.platform in ("cygwin", "win32"):
        return WindowsPipeline()
    else:
        raise Exception(f"Optimized build is not supported for platform {sys.platform}")


def gather_llvm_profiles(pipeline: Pipeline, runner: BenchmarkRunner):
    LOGGER.info("Running benchmarks with PGO instrumented LLVM")

    runner.run_llvm(pipeline)

    profile_path = pipeline.llvm_profile_merged_file()
    LOGGER.info(f"Merging LLVM PGO profiles to {profile_path}")
    cmd([
        pipeline.downloaded_llvm_dir() / "bin" / "llvm-profdata",
        "merge",
        "-o", profile_path,
        pipeline.llvm_profile_dir_root()
    ])

    LOGGER.info("LLVM PGO statistics")
    LOGGER.info(f"{profile_path}: {format_bytes(get_path_size(profile_path))}")
    LOGGER.info(
        f"{pipeline.llvm_profile_dir_root()}: {format_bytes(get_path_size(pipeline.llvm_profile_dir_root()))}")
    LOGGER.info(f"Profile file count: {count_files(pipeline.llvm_profile_dir_root())}")

    # We don't need the individual .profraw files now that they have been merged
    # into a final .profdata
    delete_directory(pipeline.llvm_profile_dir_root())


def gather_rustc_profiles(pipeline: Pipeline, runner: BenchmarkRunner):
    LOGGER.info("Running benchmarks with PGO instrumented rustc")

    runner.run_rustc(pipeline)

    profile_path = pipeline.rustc_profile_merged_file()
    LOGGER.info(f"Merging Rustc PGO profiles to {profile_path}")
    cmd([
        pipeline.build_artifacts() / "llvm" / "bin" / "llvm-profdata",
        "merge",
        "-o", profile_path,
        pipeline.rustc_profile_dir_root()
    ])

    LOGGER.info("Rustc PGO statistics")
    LOGGER.info(f"{profile_path}: {format_bytes(get_path_size(profile_path))}")
    LOGGER.info(
        f"{pipeline.rustc_profile_dir_root()}: {format_bytes(get_path_size(pipeline.rustc_profile_dir_root()))}")
    LOGGER.info(f"Profile file count: {count_files(pipeline.rustc_profile_dir_root())}")

    # We don't need the individual .profraw files now that they have been merged
    # into a final .profdata
    delete_directory(pipeline.rustc_profile_dir_root())


def gather_llvm_bolt_profiles(pipeline: Pipeline, runner: BenchmarkRunner):
    LOGGER.info("Running benchmarks with BOLT instrumented LLVM")

    runner.run_bolt(pipeline)

    merged_profile_path = pipeline.llvm_bolt_profile_merged_file()
    profile_files_path = Path("/tmp/prof.fdata")
    LOGGER.info(f"Merging LLVM BOLT profiles to {merged_profile_path}")

    profile_files = sorted(glob.glob(f"{profile_files_path}*"))
    cmd([
        "merge-fdata",
        *profile_files,
    ], output_path=merged_profile_path)

    LOGGER.info("LLVM BOLT statistics")
    LOGGER.info(f"{merged_profile_path}: {format_bytes(get_path_size(merged_profile_path))}")
    LOGGER.info(
        f"{profile_files_path}: {format_bytes(get_path_prefix_size(profile_files_path))}")
    LOGGER.info(f"Profile file count: {count_files_with_prefix(profile_files_path)}")


def clear_llvm_files(pipeline: Pipeline):
    """
    Rustbuild currently doesn't support rebuilding LLVM when PGO options
    change (or any other llvm-related options); so just clear out the relevant
    directories ourselves.
    """
    LOGGER.info("Clearing LLVM build files")
    delete_directory(pipeline.build_artifacts() / "llvm")
    delete_directory(pipeline.build_artifacts() / "lld")


def print_binary_sizes(pipeline: Pipeline):
    bin_dir = pipeline.build_artifacts() / "stage2" / "bin"
    binaries = get_files(bin_dir)

    lib_dir = pipeline.build_artifacts() / "stage2" / "lib"
    libraries = get_files(lib_dir, lambda p: p.suffix == ".so")

    paths = sorted(binaries) + sorted(libraries)
    with StringIO() as output:
        for path in paths:
            path_str = f"{path.name}:"
            print(f"{path_str:<50}{format_bytes(path.stat().st_size):>14}", file=output)
        LOGGER.info(f"Rustc binary size\n{output.getvalue()}")


def print_free_disk_space(pipeline: Pipeline):
    usage = shutil.disk_usage(pipeline.opt_artifacts())
    total = usage.total
    used = usage.used
    free = usage.free

    logging.info(
        f"Free disk space: {format_bytes(free)} out of total {format_bytes(total)} ({(used / total) * 100:.2f}% used)")


def log_metrics(step: BuildStep):
    substeps: List[Tuple[int, BuildStep]] = []

    def visit(step: BuildStep, level: int):
        substeps.append((level, step))
        for child in step.children:
            visit(child, level=level + 1)

    visit(step, 0)

    output = StringIO()
    for (level, step) in substeps:
        label = f"{'.' * level}{step.type}"
        print(f"{label:<65}{step.duration:>8.2f}s", file=output)
    logging.info(f"Build step durations\n{output.getvalue()}")


def record_metrics(pipeline: Pipeline, timer: Timer):
    metrics = load_last_metrics(pipeline.metrics_path())
    if metrics is None:
        return
    llvm_steps = tuple(metrics.find_all_by_type("bootstrap::llvm::Llvm"))
    llvm_duration = sum(step.duration for step in llvm_steps)

    rustc_steps = tuple(metrics.find_all_by_type("bootstrap::compile::Rustc"))
    rustc_duration = sum(step.duration for step in rustc_steps)

    # The LLVM step is part of the Rustc step
    rustc_duration = max(0, rustc_duration - llvm_duration)

    if llvm_duration > 0:
        timer.add_duration("LLVM", llvm_duration)
    if rustc_duration > 0:
        timer.add_duration("Rustc", rustc_duration)

    log_metrics(metrics)


def run_tests(pipeline: Pipeline):
    """
    After `dist` is executed, we extract its archived components into a sysroot directory,
    and then use that extracted rustc as a stage0 compiler.
    Then we run a subset of tests using that compiler, to have a basic smoke test which checks
    whether the optimization pipeline hasn't broken something.
    """
    build_dir = pipeline.build_root() / "build"
    dist_dir = build_dir / "dist"

    def extract_dist_dir(name: str) -> Path:
        target_dir = build_dir / "optimized-dist"
        target_dir.mkdir(parents=True, exist_ok=True)
        unpack_archive(dist_dir / f"{name}.tar.xz", target_dir=target_dir)
        extracted_path = target_dir / name
        assert extracted_path.is_dir()
        return extracted_path

    # Extract rustc, libstd, cargo and src archives to create the optimized sysroot
    rustc_dir = extract_dist_dir(f"rustc-nightly-{PGO_HOST}") / "rustc"
    libstd_dir = extract_dist_dir(f"rust-std-nightly-{PGO_HOST}") / f"rust-std-{PGO_HOST}"
    cargo_dir = extract_dist_dir(f"cargo-nightly-{PGO_HOST}") / "cargo"
    extracted_src_dir = extract_dist_dir("rust-src-nightly") / "rust-src"

    # We need to manually copy libstd to the extracted rustc sysroot
    shutil.copytree(
        libstd_dir / "lib" / "rustlib" / PGO_HOST / "lib",
        rustc_dir / "lib" / "rustlib" / PGO_HOST / "lib"
    )

    # Extract sources - they aren't in the `rustc-nightly-{host}` tarball, so we need to manually copy libstd
    # sources to the extracted sysroot. We need sources available so that `-Zsimulate-remapped-rust-src-base`
    # works correctly.
    shutil.copytree(
        extracted_src_dir / "lib" / "rustlib" / "src",
        rustc_dir / "lib" / "rustlib" / "src"
    )

    rustc_path = rustc_dir / "bin" / f"rustc{pipeline.executable_extension()}"
    assert rustc_path.is_file()
    cargo_path = cargo_dir / "bin" / f"cargo{pipeline.executable_extension()}"
    assert cargo_path.is_file()

    # Specify path to a LLVM config so that LLVM is not rebuilt.
    # It doesn't really matter which LLVM config we choose, because no sysroot will be compiled.
    llvm_config = pipeline.build_artifacts() / "llvm" / "bin" / f"llvm-config{pipeline.executable_extension()}"
    assert llvm_config.is_file()

    config_content = f"""profile = "user"
changelog-seen = 2

[build]
rustc = "{rustc_path.as_posix()}"
cargo = "{cargo_path.as_posix()}"

[target.{PGO_HOST}]
llvm-config = "{llvm_config.as_posix()}"
"""
    logging.info(f"Using following `config.toml` for running tests:\n{config_content}")

    # Simulate a stage 0 compiler with the extracted optimized dist artifacts.
    with open("config.toml", "w") as f:
        f.write(config_content)

    args = [
        sys.executable,
        pipeline.checkout_path() / "x.py",
        "test",
        "--stage", "0",
        "tests/assembly",
        "tests/codegen",
        "tests/codegen-units",
        "tests/incremental",
        "tests/mir-opt",
        "tests/pretty",
        "tests/run-pass-valgrind",
        "tests/ui",
        ]
    for test_path in pipeline.skipped_tests():
        args.extend(["--exclude", test_path])
    cmd(args=args, env=dict(
        COMPILETEST_FORCE_STAGE0="1"
    ))


def execute_build_pipeline(timer: Timer, pipeline: Pipeline, runner: BenchmarkRunner, dist_build_args: List[str]):
    # Clear and prepare tmp directory
    shutil.rmtree(pipeline.opt_artifacts(), ignore_errors=True)
    os.makedirs(pipeline.opt_artifacts(), exist_ok=True)

    pipeline.build_rustc_perf()

    """
    Stage 1: Build PGO instrumented rustc

    We use a normal build of LLVM, because gathering PGO profiles for LLVM and `rustc` at the same time
    can cause issues.
    """
    with timer.section("Stage 1 (rustc PGO)") as stage1:
        with stage1.section("Build PGO instrumented rustc and LLVM") as rustc_pgo_instrument:
            bootstrap_build(pipeline, args=[
                "--rust-profile-generate",
                pipeline.rustc_profile_dir_root()
            ])
            record_metrics(pipeline, rustc_pgo_instrument)

        with stage1.section("Gather profiles"):
            gather_rustc_profiles(pipeline, runner)
        print_free_disk_space(pipeline)

        with stage1.section("Build PGO optimized rustc") as rustc_pgo_use:
            bootstrap_build(pipeline, args=[
                "--rust-profile-use",
                pipeline.rustc_profile_merged_file()
            ])
            record_metrics(pipeline, rustc_pgo_use)
        dist_build_args += [
            "--rust-profile-use",
            pipeline.rustc_profile_merged_file()
        ]

    """
    Stage 2: Gather LLVM PGO profiles
    """
    with timer.section("Stage 2 (LLVM PGO)") as stage2:
        # Clear normal LLVM artifacts
        clear_llvm_files(pipeline)

        with stage2.section("Build PGO instrumented LLVM") as llvm_pgo_instrument:
            bootstrap_build(pipeline, args=[
                "--llvm-profile-generate",
                # We want to keep the already built PGO-optimized `rustc`.
                "--keep-stage", "0",
                "--keep-stage", "1"
            ], env=dict(
                LLVM_PROFILE_DIR=str(pipeline.llvm_profile_dir_root() / "prof-%p")
            ))
            record_metrics(pipeline, llvm_pgo_instrument)

        with stage2.section("Gather profiles"):
            gather_llvm_profiles(pipeline, runner)

        dist_build_args += [
            "--llvm-profile-use",
            pipeline.llvm_profile_merged_file(),
        ]
        print_free_disk_space(pipeline)

        # Clear PGO-instrumented LLVM artifacts
        clear_llvm_files(pipeline)

    """
    Stage 3: Build BOLT instrumented LLVM

    We build a PGO optimized LLVM in this step, then instrument it with BOLT and gather BOLT profiles.
    Note that we don't remove LLVM artifacts after this step, so that they are reused in the final dist build.
    BOLT instrumentation is performed "on-the-fly" when the LLVM library is copied to the sysroot of rustc,
    therefore the LLVM artifacts on disk are not "tainted" with BOLT instrumentation and they can be reused.
    """
    if pipeline.supports_bolt():
        with timer.section("Stage 3 (LLVM BOLT)") as stage3:
            with stage3.section("Build BOLT instrumented LLVM") as llvm_bolt_instrument:
                bootstrap_build(pipeline, args=[
                    "--llvm-profile-use",
                    pipeline.llvm_profile_merged_file(),
                    "--llvm-bolt-profile-generate",
                    # We want to keep the already built PGO-optimized `rustc`.
                    "--keep-stage", "0",
                    "--keep-stage", "1"
                ])
                record_metrics(pipeline, llvm_bolt_instrument)

            with stage3.section("Gather profiles"):
                gather_llvm_bolt_profiles(pipeline, runner)

            dist_build_args += [
                "--llvm-bolt-profile-use",
                pipeline.llvm_bolt_profile_merged_file()
            ]
            print_free_disk_space(pipeline)

    # We want to keep the already built PGO-optimized `rustc`.
    dist_build_args += [
        "--keep-stage", "0",
        "--keep-stage", "1"
    ]

    """
    Final stage: Build PGO optimized rustc + PGO/BOLT optimized LLVM
    """
    with timer.section("Final stage (dist build)") as final_stage:
        cmd(dist_build_args)
        record_metrics(pipeline, final_stage)

    # Try builds can be in various broken states, so we don't want to gatekeep them with tests
    if not is_try_build():
        with timer.section("Run tests"):
            run_tests(pipeline)


def run(runner: BenchmarkRunner):
    logging.basicConfig(
        level=logging.DEBUG,
        format="%(name)s %(levelname)-4s: %(message)s",
    )

    LOGGER.info(f"Running multi-stage build using Python {sys.version}")
    LOGGER.info(f"Environment values\n{pprint.pformat(dict(os.environ), indent=2)}")

    build_args = sys.argv[1:]

    # Skip components that are not needed for try builds to speed them up
    if is_try_build():
        LOGGER.info("Skipping building of unimportant components for a try build")
        for target in ("rust-docs", "rustc-docs", "rust-docs-json", "rust-analyzer",
                       "rustc-src", "clippy", "miri", "rustfmt"):
            build_args.extend(["--exclude", target])

    timer = Timer()
    pipeline = create_pipeline()

    try:
        execute_build_pipeline(timer, pipeline, runner, build_args)
    except BaseException as e:
        LOGGER.error("The multi-stage build has failed")
        raise e
    finally:
        timer.print_stats()
        print_free_disk_space(pipeline)

    print_binary_sizes(pipeline)


if __name__ == "__main__":
    runner = DefaultBenchmarkRunner()
    run(runner)
