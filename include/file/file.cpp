#include "file.hpp"

#include <filesystem>
#include <fstream>

using std::cout;
using std::endl;
using std::filesystem::directory_iterator;
using std::filesystem::exists;
using std::fstream;
using std::hash;
using std::ios;

vector<string> getDirs(string path) {
	vector<string> result;
	for (auto& dir : directory_iterator(path)) {
		if (!dir.is_directory()) continue;
		result.push_back(dir.path().filename().string());
	}
	return result;
}

string openFile(string path) {
    string result;
	string temp = "";
    fstream fs;

	fs.open(path, ios::in);
	if (!fs.is_open()){
		cout << "[Error] openFile: cannot open file." << endl;
		return "";
	}
	while (getline(fs, temp)) {
		if (!(result.empty()))
			result.append("\n");
		result.append(temp);
	}
	// cout << "[info] openFile: text:\n" << text.c_str() << "\n";
	fs.close();
    
    return result;
}

bool isSame(string path, string name, string sha1Str) {
	// if (!exists(path)) return false;
	// string text = openFile(path);
	// hash<string> hashFunc(string text);
	// if () return true;
	// else return false;
}

bool saveFile(string path, string content) {
    fstream fs;

    fs.open(path, ios::out);
	if (!fs){
		cout << "[Error] openFile: cannot open file. \n";
		return false;
	}
	fs << content;
	fs.close();

    return true;
}
