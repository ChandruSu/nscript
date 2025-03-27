import sys
import time

N = int(sys.argv[1])

t0 = time.perf_counter()

sum_ = 0
while N > 0:
  sum_ += N
  N -= 1

print("Total sum: " + str(sum_))

t1 = time.perf_counter()

print("Execution took (ms) " + str((t1 - t0) * 1000))