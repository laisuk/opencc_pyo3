from python.opencc_pyo3 import OpenCC

text = "“春眠不觉晓，处处闻啼鸟。夜来风雨声，花落知多少”"
opencc = OpenCC("s2twp")
converted = opencc.convert(text, True)
print(f"Original text: {text}")
print(f"Text code: {opencc.zho_check(text)}")
print(opencc.config)
print(f"Converted text: {converted}")
print(f"Text code: {opencc.zho_check(converted)}")
opencc.config = "s2hk"
print(opencc.config)
