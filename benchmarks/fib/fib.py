import sys
import time

N = int(sys.argv[1])
t0 = time.perf_counter()

def fib(n):
  if n == 1:
    return 0
  elif n < 4:
    return 1
  else:
    return fib(n - 1) + fib(n - 2)

print(fib(N))

t1 = time.perf_counter()

print("Execution took (ms) " + str((t1 - t0) * 1000))