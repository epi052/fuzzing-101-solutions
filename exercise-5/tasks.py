from pathlib import Path

from invoke import task

PROJ_DIR = Path(__file__).parent
XML_DIR = PROJ_DIR / "libxml2"
BUILD_DIR = PROJ_DIR / "build"


def run(ctx, cmd, workdir=None, hide=False):
    """execute the given command"""
    if workdir is not None:
        with ctx.cd(workdir):
            return ctx.run(cmd, pty=True, hide=hide)

    return ctx.run(cmd, pty=True, hide=hide)


@task
def build_xml(ctx, force=False):
    """download and compile libxml2"""
    if not XML_DIR.exists():
        run(ctx, "wget http://xmlsoft.org/download/libxml2-2.9.4.tar.gz")
        run(ctx, "tar xf libxml2-2.9.4.tar.gz")
        run(ctx, f"mv libxml2-2.9.4 {XML_DIR}")
        run(ctx, "rm libxml2-2.9.4.tar.gz")

    if not BUILD_DIR.exists() or force:
        BUILD_DIR.mkdir(parents=True, exist_ok=True)

        cmd = (
            f"./configure --prefix={BUILD_DIR} --disable-shared --without-debug --without-ftp"
            f" --without-http --without-legacy --without-python LIBS='-ldl'"
        )

        run(ctx, cmd, workdir=XML_DIR)
        run(ctx, "make -j $(nproc)", workdir=XML_DIR)
        run(ctx, "make install", workdir=XML_DIR)


@task
def build_afl(ctx, force=False):
    """compile pylibafl and install it using pip"""
    # commit 03c020f4bdddbcef6b5cd2c50cd8f88f9b20c3b6
    pylib = "../LibAFL/bindings/pylibafl"

    result = ctx.run("pip freeze", hide=True)

    if "pylibafl-0.7.0-cp39-cp39-linux_x86_64.whl" not in result.stdout or force:
        run(ctx, "maturin build --release", workdir=pylib)
        run(
            ctx,
            "pip install --force-reinstall target/wheels/pylibafl-0.7.0-cp39-cp39-linux_x86_64.whl",
            workdir=pylib,
        )


@task(pre=[build_xml])
def build_harness(ctx):
    """compile harness.c; store result in build/"""
    include = "-I $(pwd)/build/include/libxml2"
    links = "-L $(pwd)/build/lib/ -lxml2 -lm -llzma -lz"
    run(
        ctx,
        f"gcc -static -o harness harness.c {include} {links}",
    )
    run(ctx, "mv harness build/")


@task
def clean(ctx):
    """remove build/ directory"""
    run(ctx, f"rm -rf {BUILD_DIR}")


@task(pre=[build_harness, build_afl])
def build(_ctx):
    """call clean then build"""
    ...


@task(pre=[clean, build])
def rebuild(_ctx):
    """call clean then build"""
    ...
