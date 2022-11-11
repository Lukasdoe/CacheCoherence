import os

import matplotlib.pyplot as plt
from matplotlib.ticker import ScalarFormatter
import pandas as pd


path = os.path.dirname(os.path.abspath(__file__))
root_path = os.path.abspath(os.path.join(path, ".."))
style_path = os.path.join(path, "stylesheet.txt")
out_path = os.path.join(root_path, "report/figures")
plt.style.use(style_path)

MARGIN = 1000


class MyScalarFormatter(ScalarFormatter):
    def _set_format(self):
        self.format = "%.2f"


def cycles(df, field, legend, advanced=False):
    dfs = []
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    cond = (df["protocol"] != "Dragon") if advanced else (df["protocol"] != "Mesi (advanced)")
    for file in inputs:
        df_file = (
            df[(df["input"] == file) & cond]
            .groupby([field, "protocol"])[["total_cycles"]]
            .mean()
            .unstack("protocol", fill_value=0)
        )
        dfs.append(df_file)

    names = ["blackscholes", "bodytrack", "fluidanimate"]
    legend = ["Mesi", "Mesi (advanced)", "Difference"] if advanced else ["Dragon", "Mesi", "Difference"]
    for i in range(3):
        fig = plt.figure()
        gs = fig.add_gridspec(2, hspace=0.20)
        axs = gs.subplots(sharex=True, sharey=False)
        axs[0].axhline(y=0, color="black", linestyle="-")
        cpy = dfs[i].copy()["total_cycles"]
        if advanced:
            cpy["difference"] = cpy["Mesi"] - cpy["Mesi (advanced)"]
        else:
            cpy["difference"] = cpy["Mesi"] - cpy["Dragon"]
        cpy["difference"].plot.bar(rot=0, ax=axs[0], alpha=0.7, color=["olive"])
        axs[0].set_ylabel(r"Cycles", rotation="horizontal")
        axs[0].set_xlabel(r"")
        axs[0].legend([])
        axs[0].ticklabel_format(style="sci", scilimits=(0, 0), axis="y")
        axs[0].yaxis.set_major_formatter(MyScalarFormatter(useMathText=True))
        axs[0].yaxis.major.formatter.set_powerlimits((0, 2))
        axs[0].yaxis.set_label_coords(-0.1, 1.08)

        dfs[i].plot.bar(rot=0, ax=axs[1], alpha=0.7, color=["red", "orange", "olive"])
        axs[1].set_ylabel(r"")
        axs[1].set_xlabel(r"")
        axs[1].legend([])
        axs[1].yaxis.set_major_formatter(MyScalarFormatter(useMathText=True))
        axs[1].yaxis.major.formatter.set_powerlimits((0, 2))

        handles, _ = axs[0].get_legend_handles_labels()
        handles2, _ = axs[1].get_legend_handles_labels()
        h = handles2 + handles
        ncol = 2 if advanced else 3
        fig.legend(
            labels=legend,
            handles=h,
            loc="upper center",
            bbox_to_anchor=(0.5, 0.00),
            ncol=ncol,
        )
        advanced_name = "_advanced" if advanced else ""
        out = os.path.join(out_path, f"{field}_{names[i]}{advanced_name}.svg")
        plt.savefig(out, bbox_inches="tight")
        plt.show()


def custom(df, field, advanced=False):
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    cond = (df["protocol"] != "Dragon") if advanced else (df["protocol"] != "Mesi (advanced)")
    df = df[cond].groupby(["protocol", "input"])[[field]].mean().unstack("protocol", fill_value=0)

    names = ["Blackscholes", "Bodytrack", "Fluidanimate"]
    legend = ["Mesi", "Mesi (advanced)", "Difference"] if advanced else ["Dragon", "Mesi", "Difference"]

    fig = plt.figure()
    gs = fig.add_gridspec(2, hspace=0.20)
    axs = gs.subplots(sharex=True, sharey=False)
    axs[0].axhline(y=0, color="black", linestyle="-")
    cpy = df.copy()
    cpy = df.copy()[field]
    if advanced:
        cpy["difference"] = cpy["Mesi"] - cpy["Mesi (advanced)"]
    else:
        cpy["difference"] = cpy["Mesi"] - cpy["Dragon"]
    cpy["difference"].plot.bar(rot=0, ax=axs[0], alpha=0.7, color=["olive"])
    axs[0].set_ylabel(r"Cycles", rotation="horizontal")
    axs[0].set_xlabel(r"")
    axs[0].legend([])
    axs[0].ticklabel_format(style="sci", scilimits=(0, 0), axis="y")
    axs[0].yaxis.set_major_formatter(MyScalarFormatter(useMathText=True))
    axs[0].yaxis.major.formatter.set_powerlimits((0, 2))
    axs[0].yaxis.set_label_coords(-0.1, 1.08)

    df.plot.bar(rot=0, ax=axs[1], alpha=0.7, color=["red", "orange", "olive"])
    axs[1].set_ylabel(r"")
    axs[1].set_xlabel(r"")
    axs[1].set_xticklabels(names)
    axs[1].legend([])
    axs[1].ticklabel_format(style="sci", scilimits=(0, 0), axis="y")
    axs[1].yaxis.set_major_formatter(MyScalarFormatter(useMathText=True))
    axs[1].yaxis.major.formatter.set_powerlimits((0, 2))

    handles, _ = axs[0].get_legend_handles_labels()
    handles2, _ = axs[1].get_legend_handles_labels()
    h = handles2 + handles
    ncol = 2 if advanced else 3
    fig.legend(
        labels=legend,
        handles=h,
        loc="upper center",
        bbox_to_anchor=(0.5, 0.00),
        ncol=ncol,
    )
    advanced_name = "_advanced" if advanced else ""
    out = os.path.join(out_path, f"{field}{advanced_name}.svg")
    plt.savefig(out, bbox_inches="tight")
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
    advanced = True
    cache_size(advanced)
    block_size(advanced)
    associativity(advanced)
    bus_traffic(advanced)
    invalidations(advanced)
