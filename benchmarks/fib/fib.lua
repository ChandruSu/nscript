
local t0 = os.clock()
local N = tonumber(arg[1])

function fib(n)
  if n == 1 then
    return 0
  elseif n < 4 then 
    return 1
  else 
    return fib(n - 1) + fib(n - 2)
  end
end

print(fib(N))

local t1 = os.clock()

print("Execution took (ms) " .. ((t1 - t0) * 1000))