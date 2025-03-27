local clock = os.clock

local t0 = clock()

local N = tonumber(arg[1])
local x = {}

local i = 0
while i < N do
    x[i] = true
    i = i + 1
end

i = 2
while i * i < N do
    if x[i] then
        local j = 2 * i
        while j < N do
            x[j] = false
            j = j + i
        end
    end
    i = i + 1
end

print("Primes until: " .. N)
i = 0
while i < N do
    if x[i] then
        print(i)
    end
    i = i + 1
end

local t1 = clock()
print(string.format("Execution took (ms) " .. (t1 - t0) * 1000))
