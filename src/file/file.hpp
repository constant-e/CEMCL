#pragma once

#include <iostream>
#include <vector>

using std::string;
using std::vector;

vector<string> getDirs(string path);
bool isSame(string path, string name, string sha1Str);
string openFile(string path);
bool saveFile(string path, string content);
