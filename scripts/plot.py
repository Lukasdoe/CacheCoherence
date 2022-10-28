import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

# name = "associativity.svg"
name = "cache_size.svg"
path = os.path.dirname(os.path.abspath(__file__))
root_path = os.path.abspath(os.path.join(path, ".."))
data_path = os.path.join(root_path, "data/data.csv")
style_path = os.path.join(path, "stylesheet.txt")
out_path = os.path.join(root_path, "report/figures")
out = os.path.join(out_path, name)
plt.style.use(style_path)

df = pd.read_csv(data_path)


def plot(field):
    df_protocol_group = df.groupby(["protocol", field])[["total_cycles"]].mean().unstack(field, fill_value=0)

    fig, ax = plt.subplots()
    df_protocol_group.plot.bar(rot=0, ax=ax)
    ax.legend(["AAA", "BBB", "CCC"])
    ax.set_ylabel(r"Cycles")
    ax.set_xlabel(r"Protocol")
    plt.savefig(out)
    plt.show()


base = os.path.splitext(name)[0]
plot(base)
