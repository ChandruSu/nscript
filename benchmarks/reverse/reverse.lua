
local t0 = os.clock()
local N = tonumber(arg[1])

arr = {}

local i = 0
while i < N do
    table.insert(arr, i)
    i = i + 1
end

i = 0
while i < (N / 2) do
    temp = arr[i + 1]
    arr[i + 1] = arr[N - i]
    arr[N - i] = temp
    i = i + 1
end

local t1 = os.clock()

print("Execution took (ms) " .. ((t1 - t0) * 1000))