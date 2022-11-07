import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os


# legend = ["1024 B", "4096 B", "8192 B"]

path = os.path.dirname(os.path.abspath(__file__))
root_path = os.path.abspath(os.path.join(path, ".."))
style_path = os.path.join(path, "stylesheet.txt")
out_path = os.path.join(root_path, "report/figures")
plt.style.use(style_path)


def cycles_same(df, field, legend):
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    dfs = []
    for file in inputs:
        df_file = (
            df[(df["input"] == file) & (df["protocol"] != "Mesi (advanced)")]
            .groupby(["protocol", field])[["total_cycles"]]
            .mean()
            .unstack(field, fill_value=0)
        )
        dfs.append(df_file)

    names = ["blackscholes", "bodytrack", "fluidanimate"]
    for i in range(1):
        fig, ax = plt.subplots()
        dfs[i].plot.bar(
            rot=0,
            ax=ax,
            alpha=0.7,
            color=["red", "orange", "olive"],
        )
        ax.set_ylabel(r"Cycles")
        ax.set_xlabel(r"")
        ax.legend(legend, loc="upper center", bbox_to_anchor=(0.5, -0.08), ncol=3)
        out = os.path.join(out_path, f"{field}_{names[i]}.svg")
        plt.savefig(out)
        plt.show()


def cycles(df, field, legend):
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    dfs = []
    for file in inputs:
        df_file = (
            df[(df["input"] == file) & (df["protocol"] != "Mesi (advanced)")]
            .groupby([field, "protocol"])[["total_cycles"]]
            .mean()
            .unstack("protocol", fill_value=0)
        )
        dfs.append(df_file)

    names = ["blackscholes", "bodytrack", "fluidanimate"]
    legend = ["Dragon", "Mesi"]
    for i in range(3):
        fig, ax = plt.subplots()
        dfs[i].plot.bar(rot=0, ax=ax, alpha=0.7, color=["red", "orange", "olive"])
        ax.set_ylabel(r"Cycles")
        ax.set_xlabel(r"")
        ax.legend(legend, loc="upper center", bbox_to_anchor=(0.5, -0.08), ncol=3)
        out = os.path.join(out_path, f"{field}_{names[i]}.svg")
        plt.savefig(out)
        plt.show()


def custom(df, field):
    dfs = []
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    for file in inputs:
        df_file = (
            df[df["protocol"] != "Mesi (advanced)"]
            .groupby(["protocol", "input"])[[field]]
            .mean()
            .unstack("input", fill_value=0)
        )
        dfs.append(df_file)

    names = ["blackscholes", "bodytrack", "fluidanimate"]
    legend = ["Blackscholes", "Bodytrack", "Fluidanimate"]
    for i in range(1):
        fig, ax = plt.subplots()
        dfs[i].plot.bar(
            rot=0,
            ax=ax,
            alpha=0.7,
            color=["red", "orange", "olive"],
        )
        ax.set_ylabel(r"Cycles")
        ax.set_xlabel(r"")
        ax.legend(legend, loc="upper center", bbox_to_anchor=(0.5, -0.08), ncol=2)
        out = os.path.join(out_path, f"{field}.svg")
        plt.savefig(out)
        plt.show()


def cache_size():
    name = "cache_size"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    legend = ["1024 B", "4096 B", "8192 B"]
    cycles(df, name, legend)


def block_size():
    name = "block_size"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    legend = ["16 B", "32 B", "64 B"]
    cycles(df, name, legend)


def associativity():
    name = "associativity"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    legend = ["1", "2", "128 (full)"]
    cycles(df, name, legend)


def bus_traffic():
    name = "default"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    custom(df, "traffic")


def invalidations():
    name = "default"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    custom(df, "invalidations")


# cycles(name)
# custom("traffic")


if __name__ == "__main__":
    cache_size()
    # block_size()
    # associativity()
    # bus_traffic()
    # invalidations()
