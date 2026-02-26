if path_params[1] == "json" then
    response.headers["content-type"] = "application/json"
    return '{"hi": "hello!","bye": "cya"}'
elseif path_params[1] == "teapot" then
    response.statuscode = 418
    return "Im a teapot! (Check the Network tab on F12 :D)"
end



htmlfile = io.open("./templates/demo.html", "r")
content = htmlfile:read "a"
htmlfile:close()

return content
