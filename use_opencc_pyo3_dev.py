from opencc_pyo3 import OpenCC

# text = "“春眠不觉晓，处处闻啼鸟。夜来风雨声，花落知多少”"
text = "潦水盡而寒潭清，煙光凝而暮山紫。儼驂騑於上路，訪風景於崇阿；臨帝子之長洲，得天人之舊館。"
opencc_dev = OpenCC()
print(f"Original text: {text}")
original_text_code = opencc_dev.zho_check(text)
print(f"Original text code: {original_text_code}")
opencc_dev.config = "t2s" if original_text_code == 1 else "s2t"
converted = opencc_dev.convert(text, True)
converted_code = opencc_dev.zho_check(converted)
print(f"Auto config: {opencc_dev.config}")
print(f"Converted text: {converted}")
print(f"Converted text code: {converted_code}")
opencc_dev.config = "t2s" if converted_code == 1 else "s2t"
converted_2 = opencc_dev.convert(converted, True)
converted_2_code = opencc_dev.zho_check(converted_2)
print("Reconvert " + opencc_dev.config + ": " + converted_2)
print(f"Reconverted text code: {converted_2_code}")
