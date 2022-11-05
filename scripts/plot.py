import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os


name = "associativity"
# legend = ["1024 B", "4096 B", "8192 B"]
legend = ["1", "2", "128 (full)"]

path = os.path.dirname(os.path.abspath(__file__))
root_path = os.path.abspath(os.path.join(path, ".."))
data_path = os.path.join(root_path, f"data/{name}.csv")
style_path = os.path.join(path, "stylesheet.txt")
out_path = os.path.join(root_path, "report/figures")
plt.style.use(style_path)


df = pd.read_csv(data_path)


def cycles(field):
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    dfs = []
    for file in inputs:
        df_file = (
            df[df["input"] == file].groupby(["protocol", field])[["total_cycles"]].mean().unstack(field, fill_value=0)
        )
        dfs.append(df_file)

    names = ["blackscholes", "bodytrack", "fluidanimate"]
    for i in range(3):
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
        out = os.path.join(out_path, f"{name}_{names[i]}.svg")
        plt.savefig(out)
        plt.show()


def custom(field):
    dfs = []
    inputs = ["blackscholes_four.zip", "bodytrack_four.zip", "fluidanimate_four.zip"]
    for file in inputs:
        df_file = df.groupby(["protocol", "input"])[[field]].mean().unstack("input", fill_value=0)
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


# cycles(name)
custom("traffic")
