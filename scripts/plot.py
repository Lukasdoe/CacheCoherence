import os

import matplotlib.pyplot as plt
import pandas as pd

# legend = ["1024 B", "4096 B", "8192 B"]

path = os.path.dirname(os.path.abspath(__file__))
root_path = os.path.abspath(os.path.join(path, ".."))
style_path = os.path.join(path, "stylesheet.txt")
out_path = os.path.join(root_path, "report/figures")
plt.style.use(style_path)


def cycles_same(df, field, legend, advanced=False):
    dfs = []
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    cond = (
        (df["protocol"] != "Dragon")
        if advanced
        else (df["protocol"] != "Mesi (advanced)")
    )
    for file in inputs:
        df_file = (
            df[(df["input"] == file) & cond]
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
        advanced_name = "_advanced" if advanced else ""
        out = os.path.join(out_path, f"{field}_{names[i]}{advanced_name}.svg")
        plt.savefig(out)
        plt.show()


def cycles(df, field, legend, advanced=False):
    dfs = []
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    cond = (
        (df["protocol"] != "Dragon")
        if advanced
        else (df["protocol"] != "Mesi (advanced)")
    )
    for file in inputs:
        df_file = (
            df[(df["input"] == file) & cond]
            .groupby([field, "protocol"])[["total_cycles"]]
            .mean()
            .unstack("protocol", fill_value=0)
        )
        dfs.append(df_file)

    names = ["blackscholes", "bodytrack", "fluidanimate"]
    legend = (
        ["Mesi", "Mesi (advanced)"] if advanced else ["Dragon", "Mesi", "Difference"]
    )
    for i in range(3):
        fig, (ax1, ax2) = plt.subplots(2)
        ax1.axhline(y=0, color="black", linestyle="-")

        cpy = dfs[i].copy()["total_cycles"]
        cpy["difference"] = cpy["Mesi"] - cpy["Dragon"]
        cpy["difference"].plot.bar(rot=0, ax=ax1, alpha=0.7, color=["olive"])
        ax1.set_ylabel(r"Cycles")
        ax1.set_xlabel(r"")
        ax1.legend([])
        ax1.set_ybound(-300000, 300000)

        dfs[i].plot.bar(rot=0, ax=ax2, alpha=0.7, color=["red", "orange", "olive"])
        ax2.set_ylabel(r"Cycles")
        ax2.set_xlabel(r"")
        ax2.legend([])

        handles, _ = ax1.get_legend_handles_labels()
        handles2, _ = ax2.get_legend_handles_labels()
        h = handles + handles2
        fig.legend(
            labels=legend,
            handles=h[0:],
            loc="upper center",
            bbox_to_anchor=(0.5, 0),
            fancybox=True,
            shadow=True,
            ncol=5,
        )
        advanced_name = "_advanced" if advanced else ""
        out = os.path.join(out_path, f"{field}_{names[i]}{advanced_name}.svg")
        plt.savefig(out, bbox_inches="tight")
        plt.show()


# Ref: https://matplotlib.org/stable/gallery/subplots_axes_and_figures/subplots_demo.html
def custom(df, field, advanced=False):
    dfs = []
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    cond = (
        (df["protocol"] != "Dragon")
        if advanced
        else (df["protocol"] != "Mesi (advanced)")
    )
    for file in inputs:
        df_file = (
            df[cond]
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
        advanced_name = "_advanced" if advanced else ""
        out = os.path.join(out_path, f"{field}{advanced_name}.pdf")
        plt.savefig(out)
        plt.show()


def cache_size(advanced=False):
    name = "cache_size"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    legend = ["1024 B", "4096 B", "8192 B"]
    cycles(df, name, legend, advanced)


def block_size(advanced=False):
    name = "block_size"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    legend = ["16 B", "32 B", "64 B"]
    cycles(df, name, legend, advanced)


def associativity(advanced=False):
    name = "associativity"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    legend = ["1", "2", "128 (full)"]
    cycles(df, name, legend, advanced)


def bus_traffic(advanced=False):
    name = "default"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    custom(df, "traffic", advanced)


def invalidations(advanced=False):
    name = "default"
    data_path = os.path.join(root_path, f"data/{name}.csv")
    df = pd.read_csv(data_path)
    custom(df, "invalidations", advanced)


if __name__ == "__main__":
    cache_size(False)
    # block_size(True)
    # associativity(True)
    # bus_traffic()
    # invalidations()
