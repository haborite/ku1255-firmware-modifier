import json

with open("dev/__default__.json", "r", encoding="utf-8") as f:
    data = json.load(f)

data["layer0"] = {str(k): v for k, v in data["layer0"]}
data["layer1"] = {str(k): v for k, v in data["layer1"]}

with open("dev/__default__out.json", "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
