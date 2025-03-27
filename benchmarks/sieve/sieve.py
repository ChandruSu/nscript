import time
import sys

t0 = time.perf_counter()

N = int(sys.argv[1])
x = []

i = 0
while i < N:
    x.append(True)
    i += 1

i = 2
while i * i < N:
    if not x[i]:
        i += 1
        continue
    j = 2 * i
    while j < N:
        x[j] = False
        j += i
    i += 1

print("Primes until: " + str(N))
i = 0
while i < N:
    if x[i]:
        print(i)
    i += 1

t1 = time.perf_counter()
print("Execution took (ms) " + str((t1 - t0) * 1000))
