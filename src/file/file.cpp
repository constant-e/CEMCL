#include <filesystem>
#include <fstream>

#include "file.hpp"

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
    #ifdef DEBUG
		cout << "[info] file::openFile : Start reading " << path << "." << endl;
	#endif
	string result;
	string temp = "";
    fstream fs;
	fs.open(path, ios::in);
	if (!fs.is_open()){
		cout << "[Error] file::openFile : Can't open " << path << "." << endl;
		return "";
	}
	while (getline(fs, temp)) {
		if (!(result.empty()))
			result.append("\n");
		result.append(temp);
	}
	#ifdef DEBUG
		cout << "[info] file::openFile : Done. Result:\r" << text << endl;
	#endif
	fs.close();
    return result;
}

bool isSame(string path, string name, string sha1Str) {
	// if (!exists(path)) return false;
	// string text = openFile(path);
	// hash<string> hashFunc(string text);
	// if () return true;
	// else return false;
	return true;
}

bool saveFile(string path, string content) {
    #ifdef DEBUG
		cout << "[info] file::saveFile : Start writing to " << path
			 << ". Content:\r" << content << endl;
	#endif
	fstream fs;
    fs.open(path, ios::out);
	if (!fs){
		cout << "[Error] file::saveFile : Can't open " << path << "." << endl;
		return false;
	}
	fs << content;
	fs.close();
	#ifdef DEBUG
		cout << "[info] file::saveFile : Done." << endl;
	#endif
    return true;
}
