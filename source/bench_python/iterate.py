# > time python source/bench_python/iterate.py

import math

x = 0
for i in range(0, 100000000):
    # if i passes test
    x = math.sqrt(x + 1)
print("x " + str(x))
