hello = "Welcome to "
k = "Kob"
w = "Web"

local cjson = require("cjson")
local yaya = require("yaya")
local json = cjson.encode({
    foo = "bar",
    some_object = {},
    some_array = cjson.empty_array
})

--print(type(params[1]), type(params[2]))
--[[print(json)
print(request.method)]]
--return hello .. heythere .. "All of this is being ran on Lua using mlua as we speak kekww.\n" .. "Today it is \n" .. os.date() .. "\n" .. here you have
yo = function()
    local parts = {}
    for k, v in pairs(query_params) do
        table.insert(parts, k .. "=" .. v)
    end
    return table.concat(parts, ", ")
end
print(string.format("path_params: [%s]\nquery_params: [%s]", table.concat(path_params, ", "), yo()))

-- For the meantime while I implement template rendering, you can fetch templates with a code simillar to the one download bellow
contentfile = io.open("./templates/index.html", "r")
content = contentfile:read("a")
contentfile:close()
--This fetches as reads the contents on templates/index.html and saves in on the "content" variable

--And now we format "content" with the data we want
return string.format(content, hello, k, w, os.date(), json, table.concat(path_params, ", "), yo(), request.method)
