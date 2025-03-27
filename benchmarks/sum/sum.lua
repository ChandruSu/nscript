
local t0 = os.clock()
local N = tonumber(arg[1])

local sum = 0
while N > 0 do
    sum = sum + N
    N = N - 1
end

print("Total sum: " .. sum)

local t1 = os.clock()

print("Execution took (ms) " .. ((t1 - t0) * 1000))