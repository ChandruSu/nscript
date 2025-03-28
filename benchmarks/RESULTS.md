
# Benchmarks

All benchmarks were executed with Intel64 2.8GHz, 16GB RAM, on a Windows 11 machine. Python scripts were run with Python version 3.10.4 (64 bit) and Lua scripts were run with Lua 5.4 (32 bit).

You can run a script using one of the following commands, with a custom value for `N`:
+ Python
```sh
python benchmarks/x/x.py <N>
```
+ Lua
```sh
lua benchmarks/x/x.lua <N>
```
+ NewScript
```sh
target/release/ns run benchmarks/x/x.lua --args <N>
```

All results were executed with no additional applications running in the background besides system tasks, firewall, and the terminal and with a power source attached (not in power saver mode).

# Results

## Sieve With IO and N=200,000

Description: Uses the sieve of Eratosthenes algorithm to retrieve the first N primes and prints them to console

Tests: arrays, iteration, arithmetic, IO

Test|Python 3.10.4/ms|Lua 5.4/ms|NewScript/ms
:--:|---:|---:|---:
 1|1703|3502|2640
 2|1931|3621|2579
 3|2207|3916|2608
 4|2019|3608|2737
 5|1851|3573|2484
 6|1970|3665|2806
 7|1828|3585|2683
 8|1968|3756|2703
 9|1816|3566|2671
10|1928|3491|2654
Average|1922.1(72.4%)|3628.3(137%)|2656.5

## Sieve Without IO and N=4,000,000

Description: Uses the sieve of Eratosthenes algorithm to retrieve the first N primes without printing them to console

Tests: arrays, iteration, arithmetic

Test|Python 3.10.4/ms|Lua 5.4/ms|NewScript/ms
:--:|---:|---:|---:
 1|1861|390|1525
 2|1640|376|1520
 3|1959|349|1525
 4|1617|376|1499
 5|1904|366|1529
 6|1647|385|1568
 7|1946|376|1568
 8|1898|378|1494
 9|1556|380|1510
10|1674|397|1502
Average|1770.2(116%)|377.3(24.8%)|1524

## Sum with N=100,000,000

Description: Calculates the sum of the first N natural numbers (including 0)

Tests: arrays, iteration, arithmetic

Test|Python 3.10.4/ms|Lua 5.4/ms|NewScript/ms
:--:|---:|---:|---:
 1|10721|826|5330
 2|10693|866|4808
 3|11891|840|4664
 4|11755|852|4786
 5|10699|810|5055
 6|10216|809|5497
 7|9997 |835|4920
 8|11097|812|4641
 9|10740|841|5286
10|10887|807|4652
Average|10869.6(219%)|829.8(16.7%)|4963.9

## Fib with N=36

Description: Calculates the Nth fibonacci number without the memoization optimisation

Tests: recursion, function invocation, arithmetic

Test|Python 3.10.4/ms|Lua 5.4/ms|NewScript/ms
:--:|---:|---:|---:
 1|1737|682|1483
 2|1785|638|1441
 3|1769|715|1592
 4|1815|656|1454
 5|1980|702|1458
 6|2381|708|1571
 7|2232|712|1474
 8|2018|648|1588
 9|1926|652|1473
10|2023|750|1421
Average|1966.6(132%)|686.3(45.9%)|1495.5

## Reverse with N=10000000

Description: Creates an array from 0 to N (exclusive) and then reverses the array in place.

Tests: arrays, iteration

Test|Python 3.10.4/ms|Lua 5.4/ms|NewScript/ms
:--:|---:|---:|---:
 1|3415|1235|2945
 2|2948|1317|2905
 3|2837|1237|2780
 4|3164|1180|2758
 5|3333|1256|2854
 6|3256|1226|2744
 7|2779|1256|2858
 8|2795|1239|2960
 9|2795|1300|2757
10|3022|1220|2706
Average|3034.4(107%)|1246.6(44.1%)|2826.7

## Closure with N=10000000

Description: Calculates the sum of the first N natural numbers (including 0) by chained invocation of adder closures

Tests: closures, functions, iteration

Test|Python 3.10.4/ms|Lua 5.4/ms|NewScript/ms
:--:|---:|---:|---:
 1|3341|2663|2656
 2|3163|3150|2348
 3|3267|2875|2379
 4|3244|2657|2343
 5|3386|2681|2474
 6|3395|2595|2398
 7|3370|2655|2310
 8|3258|2604|2344
 9|3174|2691|2412
10|3308|2692|2361
Average|3290.6(137%)|2726.3(113%)|2402.5

## String

Description: Repeatedly concatenates two strings a and b, N times until the length of the final string is `fib(2N + 2)`

Tests: strings, concatenation

Test|Python 3.10.4/ms|Lua 5.4/ms|NewScript/ms
:--:|---:|---:|---:
 1|164|171|213
 2|157|155|211
 3|145|167|201
 4|144|172|209
 5|145|148|215
 6|149|148|199
 7|160|164|196
 8|146|167|202
 9|153|172|205
10|154|160|209
Average|151.7(73.6%)|162.4(78.8%)|206.0