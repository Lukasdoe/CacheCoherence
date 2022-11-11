import pandas as pd
import subprocess
import itertools
import os


# associativity
# name = "associativity.csv"
# cache_sizes = ["4096"]
# associativities = ["1", "128"]
# block_sizes = ["32"]

# cache size
# name = "cache_size.csv"
# cache_sizes = ["1024", "8192"]
# associativities = ["2"]
# block_sizes = ["32"]

# block size
# name = "block_size.csv"
# cache_sizes = ["4096"]
# associativities = ["2"]
# block_sizes = ["16", "64"]

# default
# name = "default.csv"
# cache_sizes = ["4096"]
# associativities = ["2"]
# block_sizes = ["32"]

path = os.path.dirname(os.path.abspath(__file__))
root_path = os.path.abspath(os.path.join(path, ".."))
data_path = os.path.join(root_path, "data")
blackscholes_path = os.path.join(data_path, "blackscholes_four.zip")
bodytrack_path = os.path.join(data_path, "bodytrack_four.zip")
fluidanimate_path = os.path.join(data_path, "fluidanimate_four.zip")
target = os.path.join(root_path, "target/release/coherence")
out_path = os.path.join(data_path, name)


protocols = ["mesi", "dragon", "mesi-advanced"]
inputs = [blackscholes_path, bodytrack_path, fluidanimate_path]

a = [protocols, inputs, cache_sizes, associativities, block_sizes]
all_options = list(itertools.product(*a))

columns = [
    "protocol",
    "input",
    "cache_size",
    "associativity",
    "block_size",
    "total_cycles",
    "total_private_accesses",
    "total_shared_accesses",
    "total_hits",
    "total_hits_percentage",
    "total_misses",
    "total_misses_percentage",
    "traffic",
    "invalidations",
    "core0_instructions",
    "core0_exec_cycles",
    "core0_compute_cycles",
    "core0_idle_cycles",
    "core0_memory_instructions",
    "core0_load_instructions",
    "core0_store_instructions",
    "core0_hits",
    "core0_hits_percentage",
    "core0_misses",
    "core0_misses_percentage",
    "core1_instructions",
    "core1_exec_cycles",
    "core1_compute_cycles",
    "core1_idle_cycles",
    "core1_memory_instructions",
    "core1_load_instructions",
    "core1_store_instructions",
    "core1_hits",
    "core1_hits_percentage",
    "core1_misses",
    "core1_misses_percentage",
    "core2_instructions",
    "core2_exec_cycles",
    "core2_compute_cycles",
    "core2_idle_cycles",
    "core2_memory_instructions",
    "core2_load_instructions",
    "core2_store_instructions",
    "core2_hits",
    "core2_hits_percentage",
    "core2_misses",
    "core2_misses_percentage",
    "core3_instructions",
    "core3_exec_cycles",
    "core3_compute_cycles",
    "core3_idle_cycles",
    "core3_memory_instructions",
    "core3_load_instructions",
    "core3_store_instructions",
    "core3_hits",
    "core3_hits_percentage",
    "core3_misses",
    "core3_misses_percentage",
]
df = pd.DataFrame(columns=columns)


def parse(lines):
    i = 0
    data = []
    while i < len(lines):
        line = lines[i]
        if line.startswith("#"):
            i += 4
            cycles = int(lines[i].split()[-1])
            i += 1
            private_accesses = int(lines[i].split()[-1])
            i += 1
            shared_accesses = int(lines[i].split()[-1])
            i += 1
            hits = int(lines[i].split()[-2])
            hits_percentage = float(lines[i].split()[-1][1:-1])
            i += 1
            misses = int(lines[i].split()[-2])
            misses_percentage = float(lines[i].split()[-1][1:-1])
            i += 1
            traffic = int(lines[i].split()[-2])
            i += 1
            invalidations = int(lines[i].split()[-1])
            data.extend(
                [
                    cycles,
                    private_accesses,
                    shared_accesses,
                    hits,
                    hits_percentage,
                    misses,
                    misses_percentage,
                    traffic,
                    invalidations,
                ]
            )

        if line.startswith("Core Statistics"):
            cores = "\n".join(lines[i + 1 :]).split("\n\n")
            for lines in cores[:-1]:
                lines = lines.split("\n")
                i = 1
                instructions = int(lines[i].split()[-1])
                i += 1
                exec_cycles = int(lines[i].split()[-1])
                i += 1
                comp_cycles = int(lines[i].split()[-2])
                i += 1
                idle_cycles = int(lines[i].split()[-2])
                i += 1
                memory_instructions = int(lines[i].split()[-2])
                i += 1
                load_instructions = int(lines[i].split()[-2])
                i += 1
                store_instructions = int(lines[i].split()[-2])
                i += 1
                hits = int(lines[i].split()[-2])
                hits_percentage = float(lines[i].split()[-1][1:-1])
                i += 1
                misses = int(lines[i].split()[-2])
                misses_percentage = float(lines[i].split()[-1][1:-1])

                data.extend(
                    [
                        instructions,
                        exec_cycles,
                        comp_cycles,
                        idle_cycles,
                        memory_instructions,
                        load_instructions,
                        store_instructions,
                        hits,
                        hits_percentage,
                        misses,
                        misses_percentage,
                    ]
                )
            return data
        i += 1


for i, options in enumerate(all_options):
    protocol, input_file, cache_size, associativity, block_size = options
    input_name = os.path.basename(input_file)

    print(f"[{i+1}/{len(all_options)}] {options}")
    if protocol == "mesi-advanced":
        p = subprocess.Popen(
            [target, "mesi", input_file, cache_size, associativity, block_size, "--no-progress", "--read-broadcast"],
            stdout=subprocess.PIPE,
        )
    else:
        p = subprocess.Popen([target, *options, "--no-progress"], stdout=subprocess.PIPE)

    out, err = p.communicate()
    lines = out.decode("utf-8").split("\n")
    df_data = pd.DataFrame(
        [[protocol, input_name, cache_size, associativity, block_size, *parse(lines)]], columns=columns
    )
    df = pd.concat([df, df_data])
    df.to_csv(out_path, index=False)
