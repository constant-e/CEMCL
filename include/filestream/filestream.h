#ifndef filestream_H
#define filestream_H

#include <fstream>
#include <iostream>
#include <vector>

using namespace std;

string openFile(string path)
{
    string result;
	string temp = "";
    fstream fs;

	fs.open(path, ios::in);
	if (!fs.is_open()){
		cout << "[Error] openFile: cannot open file. \n";
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

bool saveFile(string path, string content)
{
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

#endif // filestream_H
