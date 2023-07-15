# A script that translates an ASCII text file into brainfuck code that generates that text
# PYINSTALLER ARGS: pyinstaller text_2_brainfuck.py --clean --name "text_2_brainfuck" --console --onefile --noconfirm --icon=NONE


import os
import sys


def gen_mul_table():
	mul_table = [[100, 100, i] for i in range(128)]
	for i in range(1, 128):
		for j in range(1, 128):
			if i * j < 128:
				if (i + j) < (mul_table[i * j][0] + mul_table[i * j][1]) and i != 1 and j != 1:
					mul_table[i * j] = [i, j, i * j]
			else:
				break
	deleted = 0
	for i in range(len(mul_table)):
		if mul_table[i - deleted][0] == 100 and mul_table[i - deleted][1] == 100:
			del mul_table[i - deleted]
			deleted += 1
	deleted = 0
	for i in range(1, len(mul_table)):
		if mul_table[i - deleted][0] + mul_table[i - deleted][1] >= \
				mul_table[i - deleted - 1][0] + mul_table[i - deleted - 1][1] + mul_table[i - deleted][2] - \
				mul_table[i - deleted - 1][2]:
			del mul_table[i - deleted]
			deleted += 1
	deleted = 0
	for i in range(0, len(mul_table) - 1, -1):
		if mul_table[i - deleted][0] + mul_table[i - deleted][1] >= \
				mul_table[i - deleted + 1][0] + mul_table[i - deleted + 1][1] + mul_table[i - deleted][2] - \
				mul_table[i - deleted + 1][2]:
			del mul_table[i - deleted]
			deleted += 1
	deleted = 0
	for i in range(len(mul_table)):
		if mul_table[i - deleted][2] <= 10:
			del mul_table[i - deleted]
			deleted += 1
	return mul_table

def is_ascii(s):
	return all(0 <= ord(c) < 128 for c in s)

def store_brainfuck_chars(chars, mul_table):
	brainfuck_code = []

	mul_results = [x[2] for x in mul_table]
	for character in chars:
		char_val = ord(character)
		if char_val <= 10:
			for _ in range(char_val):
				brainfuck_code.append("+")
			brainfuck_code.append(">")
			continue

		higher = lower = char_val
		while higher not in mul_results or lower not in mul_results:
			higher += 1
			lower -= 1

		if higher in mul_results:
			if lower in mul_results:
				i = 0
				while mul_table[i][2] != higher:
					i += 1
				factors_high = (mul_table[i][0], mul_table[i][1])
				i = 0
				while mul_table[i][2] != lower:
					i += 1
				factors_low = (mul_table[i][0], mul_table[i][1])
				if sum(factors_high) > sum(factors_low):
					factors = factors_high
				else:
					factors = factors_low
			else:
				i = 0
				while mul_table[i][2] != higher:
					i += 1
				factors = (mul_table[i][0], mul_table[i][1])
		else:
			i = 0
			while mul_table[i][2] != lower:
				i += 1
			factors = (mul_table[i][0], mul_table[i][1])

		brainfuck_code.append(">")
		for _ in range(factors[0]):
			brainfuck_code.append("+")
		brainfuck_code.append("[")
		brainfuck_code.append("<")
		for _ in range(factors[1]):
			brainfuck_code.append("+")
		brainfuck_code.append(">")
		brainfuck_code.append("-")
		brainfuck_code.append("]")

		difference = char_val - factors[0] * factors[1]
		if difference == 0:
			continue
		elif difference > 0:
			brainfuck_code.append("<")
			for _ in range(difference):
				brainfuck_code.append("+")
			brainfuck_code.append(">")
		else:
			brainfuck_code.append("<")
			for _ in range(abs(difference)):
				brainfuck_code.append("-")
			brainfuck_code.append(">")

	return brainfuck_code

def print_brainfuck_text(brainfuck_code, text, chars, position):
	for character in text:
		character_index = chars.index(character)
		difference = character_index - position
		if difference == 0:
			pass
		elif difference > 0:
			for _ in range(difference):
				brainfuck_code.append(">")
		else:
			for _ in range(abs(difference)):
				brainfuck_code.append("<")
		brainfuck_code.append(".")
		position += difference


if __name__ == "__main__":

	app_path = ""
	if getattr(sys, 'frozen', False):
		app_path = os.path.dirname(sys.executable)
	elif __file__:
		app_path = os.path.dirname(__file__)

	mul_table = gen_mul_table()
	input_file = os.path.join(app_path, input("Enter the name of the text file: "))
	output_file = os.path.join(app_path, input("Enter the name of the output file: "))

	with open(input_file, "r") as file:
		text = file.read()

	chars = list(set(list(text)))
	assert is_ascii(chars), "The text file contains non-ASCII characters."
	chars.sort(key=lambda x: ord(x))

	brainfuck_code = store_brainfuck_chars(chars, mul_table)
	brainfuck_code.append("<")
	position = len(chars) - 1

	print_brainfuck_text(brainfuck_code, text, chars, position)

	with open(output_file, "w") as file:
		file.write("".join(brainfuck_code))
