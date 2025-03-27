import sys
import time

N = int(sys.argv[1])

t0 = time.perf_counter()

a = "a"
b = "b"
while N > 0:
  a = a + b
  b = b + a
  N -= 1


t1 = time.perf_counter()

print("Length of a: " + str(len(a)))
print("Execution took (ms) " + str((t1 - t0) * 1000))