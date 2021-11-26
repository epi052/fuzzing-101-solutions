#!/usr/bin/env python3
import argparse
import subprocess
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor


def absolute_path(unvalidated):
    """ Helper to turn relative paths to absolute and validate they exist """
    path = Path(unvalidated).resolve()

    if path.exists():
        return str(path)
    else:
        raise argparse.ArgumentTypeError(f"{str(path)} does not exist; exiting.")


def main(user_input):
    """ Kicks off N number of processes in order to run afl-tmin against the input directory """
    commands = list()

    for file in Path(user_input.input).iterdir():
        outfile = Path(user_input.output) / file.stem

        tmp_cmd = [
            user_input.afl_tmin_path,
            "-i",
            str(file),
            "-o",
            str(outfile),
            "--",
            user_input.target,
        ]

        if user_input.args:
            tmp_cmd += user_input.args

        commands.append(tmp_cmd)

    with ProcessPoolExecutor(max_workers=user_input.cores) as executor:
        executor.map(subprocess.run, commands)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument(
        "input", type=absolute_path, help="directory used as input to afl-tmin"
    )
    parser.add_argument(
        "output",
        type=absolute_path,
        help="directory to store results after running afl-tmin",
    )
    parser.add_argument("target", type=absolute_path, help="path to fuzz target")
    parser.add_argument(
        "-a",
        "--args",
        help="arguments passed to fuzz target (hint: must be last in cli)",
        nargs=argparse.REMAINDER,
    )
    parser.add_argument(
        "-c", "--cores", default=6, type=int, help="number of CPU cores to use"
    )
    parser.add_argument(
        "--afl-tmin-path",
        type=absolute_path,
        default="./afl-tmin",
        help="path to afl-tmin binary",
    )

    args = parser.parse_args()

    main(args)
