import sys
import time

N = int(sys.argv[1])

t0 = time.perf_counter()

arr = []

i = 0
while i < N:
  arr.append(i)
  i += 1

i= 0
while i < (N / 2):
  temp = arr[i]
  arr[i] = arr[N - i - 1]
  arr[N - i - 1] = temp 
  i += 1

t1 = time.perf_counter()
print("Execution took (ms) " + str((t1 - t0) * 1000))