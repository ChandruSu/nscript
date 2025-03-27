
local t0 = os.clock()
local N = tonumber(arg[1])

function adder(n)
  function add(x)
      return n + x
  end
  return add
end

local sum = 0
while N > 0 do
  sum = adder(N)(sum)
  N = N - 1
end

print("Sum: " .. sum)

local t1 = os.clock()

print("Execution took (ms) " .. ((t1 - t0) * 1000))