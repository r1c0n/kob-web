print("lmfao")
hello = "Welcome to "
heythere = "KobWeb Lua!"

local cjson = require("cjson")
local json = cjson.encode({
    foo = "bar",
    some_object = {},
    some_array = cjson.empty_array
})
print(json)
--return hello .. heythere .. "All of this is being ran on Lua using mlua as we speak kekww.\n" .. "Today it is \n" .. os.date() .. "\n" .. here you have 
return string.format(
[[
%s %s, All of this being ran on Lua using mlua as we speak kekww 
Today it is: %s
And here if you have the output of the rock CJSON being used on Kob-Web!:
%s

Request-Type: %s
]], hello,  heythere, os.date(), json, request.method)