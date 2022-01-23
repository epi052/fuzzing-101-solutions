import argparse
from dataclasses import dataclass

import lief
from pylibafl import qemu, sugar

MAX_SIZE = 0x10000  # mirrors size in harness.c


@dataclass
class Fuzzer:
    """Wrapper for QemuBytesCoverageSugar-based fuzzer"""

    target: str
    input: list[str]
    output: str
    cores: list[int]
    port: int

    def run(self):
        # create a libafl_qemu Emulator, using the x86_64 version of qemu-user as its base
        emulator = qemu.Emulator(["qemu-x86_64", self.target], [])

        # parse the target and get a pointer to our harness's entrypoint
        elf = lief.parse(self.target)
        harness_func = elf.get_function_address("LLVMFuzzerTestOneInput")

        # reserve some space for our input bytes in memory
        input_bytes = emulator.map_private(0, MAX_SIZE, qemu.mmap.ReadWrite)

        # account for PIE by adding the emulators base address to the harness entrypoint
        if elf.is_pie:
            harness_func += emulator.load_addr()

        # set a breakpoint on the entrypoint and emulate execution until we arrive there
        emulator.set_breakpoint(harness_func)
        emulator.run()

        # save off the stack pointer and return address from the point of view of the entrypoint
        rsp = emulator.read_reg(qemu.regs.Rsp)
        ret_addr = int.from_bytes(emulator.read_mem(rsp, 8), "little")

        # remove entrypoint breakpoint and place new bp at the address where we want execution to stop
        emulator.remove_breakpoint(harness_func)
        emulator.set_breakpoint(ret_addr)

        def harness(in_bytes):
            """internal harness function passed to the fuzzer, similar to a rust closure"""

            if len(in_bytes) > MAX_SIZE:
                # limit input size to what's allocated in the emulator
                in_bytes = in_bytes[:MAX_SIZE]

            # write the bytes coming into the harness to the reserved space in memory
            emulator.write_mem(input_bytes, in_bytes)

            # first arg to LLVMFuzzerTestOneInput
            emulator.write_reg(qemu.regs.Rdi, input_bytes)

            # second arg to LLVMFuzzerTestOneInput
            emulator.write_reg(qemu.regs.Rsi, len(in_bytes))

            # set the stack pointer to its location at the time of hitting the entrypoint
            emulator.write_reg(qemu.regs.Rsp, rsp)

            # set the instruction pointer to the entrypoint
            emulator.write_reg(qemu.regs.Rip, harness_func)

            # mode=go
            emulator.run()

        sugar.QemuBytesCoverageSugar(
            self.input, self.output, self.port, self.cores
        ).run(emulator, harness)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-t", "--target", default="build/harness")
    parser.add_argument("-i", "--input", default=["corpus"], nargs="+")
    parser.add_argument("-o", "--output", default="solutions")
    parser.add_argument("-c", "--cores", default=[7], nargs="+", type=int)
    parser.add_argument("-p", "--port", default=1337, type=int)

    parsed = parser.parse_args()

    fuzzer = Fuzzer(**vars(parsed))
    fuzzer.run()
