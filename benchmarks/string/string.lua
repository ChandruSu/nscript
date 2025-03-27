
local t0 = os.clock()
local N = tonumber(arg[1])

local a = "a"
local b = "b"
while N > 0 do
    a = a .. b
    b = b .. a
    N = N - 1
end

local t1 = os.clock()

print("Length of a: " .. string.len(a))

print("Execution took (ms) " .. ((t1 - t0) * 1000))