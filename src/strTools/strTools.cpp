#include <regex>

#include "strTools.hpp"

using std::regex;
using std::sregex_token_iterator;

vector<string> splitStr(const string & str, char delim) {
	string s;
	s.append(1, delim);
	regex reg(s);
	vector<string> result(sregex_token_iterator(str.begin(), str.end(), reg, -1), sregex_token_iterator());
	return result;
}

void strReplace(string * str, string oldVaule, string newValue) {
	int pos = str->find(oldVaule);
	while (pos != str->npos) {
		str->replace(str->find(oldVaule), oldVaule.length(), newValue);
		pos = str->find(oldVaule);
	}
}
