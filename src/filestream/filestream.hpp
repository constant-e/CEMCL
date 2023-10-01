#ifndef filestream_H
#define filestream_H

#include <iostream>
#include <vector>

using namespace std;

string openFile(string path);
bool saveFile(string path, string content);

#endif // filestream_H
