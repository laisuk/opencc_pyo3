from python.opencc_pyo3 import OpenCC

text = "“春眠不觉晓，处处闻啼鸟。夜来风雨声，花落知多少”"
# text = "潦水盡而寒潭清，煙光凝而暮山紫。儼驂騑於上路，訪風景於崇阿；臨帝子之長洲，得天人之舊館。"
opencc = OpenCC()
print(f"Original text: {text}")
print(f"Supported config: {OpenCC.supported_configs()}")
print(f"Default config: {opencc.config}")
original_text_code = opencc.zho_check(text)
print(f"Original text code: {original_text_code}")
opencc.config = "t2s" if original_text_code == 1 else "s2t"
converted = opencc.convert(text, True)
converted_code = opencc.zho_check(converted)
print(f"Auto config: {opencc.get_config()}")
print(f"Converted text: {converted}")
print(f"Converted text code: {converted_code}")
opencc.config = "t2s" if converted_code == 1 else "s2t"
converted_2 = opencc.convert(converted, True)
print("Reconvert " + opencc.config + ": " + converted_2)
print(f"Is \'invalid\' a valid config string? {OpenCC.is_valid_config("invalid")}")
opencc.set_config("invalid")
print("Last error: " + opencc.get_last_error())
print("Current config: " + opencc.get_config())
