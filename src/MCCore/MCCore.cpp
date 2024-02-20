#include <filesystem>
#include <set>

#include "file/file.hpp"
#include "MCCore.hpp"
#include "network/network.hpp"
#include "strTools/strTools.hpp"

using sonic_json::Document;
using std::cout;
using std::endl;
using std::filesystem::exists;
using std::set;
using std::to_string;

// 已有变量：gameDir - .minecraft位置    javaDir - java位置(可执行文件)

#ifdef _WIN32
    string os = "windows";
#elif __APPLE__
    string os = "osx";
#elif __linux__
    string os = "linux";
#else
    string os = ""
    #Warning mc_core: Your OS or compiler may not be supported.
#endif

#ifdef __aarch64__
    string arch = "aarch64";
#elif __arm__
    string arch = "arm";
#elif __x86_64__
    string arch = "x86_64";
#elif __i386__
    string arch = "x86";
#else
    string arch = "";
    #Warning mc_core: Your OS or compiler may not be supported.
#endif

string addArgs(Node &n) {
    string result = "";
    for (int i = 0; n.AtPointer(i) != nullptr; i++) {
        if (n.AtPointer(i)->IsString()) {
            // 无限制，直接添加
            result.append(n.AtPointer(i)->GetString()).append(" ");
            continue;
        }

        // 不是String，判断rules
        if (!n.AtPointer(i)->HasMember("value")) continue; // 格式错误
        if (!n.AtPointer(i)->HasMember("rules")) continue; // 格式错误
        Node * r = n.AtPointer(i, "rules");
        bool allow = true;
        for (int j = 0; r->AtPointer(j) != nullptr; j++) {
            if (!r->AtPointer(j)->HasMember("action")) {
                allow = false;
                break;
            }
            if (r->AtPointer(j)->HasMember("features")) {
                allow = false;
                break; // not support yet
            }
            if (r->AtPointer(j, "action")->GetString() == "allow") {
                if (r->AtPointer(j, "os")->HasMember("name") &&
                    r->AtPointer(j, "os", "name")->GetString() != os) {
                    allow = false;
                    break;
                }
                if (r->AtPointer(j, "os")->HasMember("arch") &&
                    r->AtPointer(j, "os", "arch")->GetString() != arch) {
                    allow = false;
                    break;
                }
            } else if (r->AtPointer(j, "action")->GetString() == "disallow") {
                if (r->AtPointer(j, "os")->HasMember("name") &&
                    r->AtPointer(j, "os", "name")->GetString() == os) {
                    allow = false;
                    break;
                }
                if (r->AtPointer(j, "os")->HasMember("arch") &&
                    r->AtPointer(j, "os", "arch")->GetString() == arch) {
                    allow = false;
                    break;
                }
            }
        }
        
        if (!allow) continue;
        if (n.AtPointer(i, "value")->IsString()) {
            // 是一条参数
            result.append(n.AtPointer(i, "value")->GetString()).append(" ");
        } else {
            // 是数组
            for (int k = 0; n.AtPointer(i, "value", k) != nullptr; k++) {
                if (!n.AtPointer(i, "value", k)->IsString()) continue; // 格式错误
                result.append(n.AtPointer(i, "value", k)->GetString()).append(" ");
            }
        }
    }
    return result;
}

bool addGame() {
    return true;
}

// TODO 下载相关
vector<string> addLibs(Node &n, string gameDir) {
    vector<string> result;
    for (int i = 0; n.AtPointer(i) != nullptr; i++) {
        if (!n.AtPointer(i)->HasMember("name")) continue; // 格式错误
        string temp = gameDir + "/libraries/";
        bool allow = true;
        if (n.AtPointer(i)->HasMember("rules")) {
            // 有rules，进行判断
            Node * r = n.AtPointer(i, "rules");
            for (int j = 0; r->AtPointer(j) != nullptr; j++) {
                if (!r->AtPointer(j)->HasMember("action")) {
                    allow = false;
                    break;
                }
                // ver < 1.13 存在{"action":"allow"}, etc.
                if (!r->AtPointer(j)->HasMember("os")) continue;
                if (r->AtPointer(j, "action")->GetString() == "allow") {
                    if (r->AtPointer(j, "os")->HasMember("name") &&
                        r->AtPointer(j, "os", "name")->GetString() != os) {
                        allow = false;
                        break;
                    }
                    if (r->AtPointer(j, "os")->HasMember("arch") &&
                        r->AtPointer(j, "os", "arch")->GetString() != arch) {
                        allow = false;
                        break;
                    }
                } else if (r->AtPointer(j, "action")->GetString() == "disallow") {
                    if (r->AtPointer(j, "os")->HasMember("name") &&
                        r->AtPointer(j, "os", "name")->GetString() == os) {
                        allow = false;
                        break;
                    }
                    if (r->AtPointer(j, "os")->HasMember("arch") &&
                        r->AtPointer(j, "os", "arch")->GetString() == arch) {
                        allow = false;
                        break;
                    }
                }
            }
        }
        if (!allow) continue;
        // name: 包名:jar名:版本
        vector<string> nameS = splitStr(n.AtPointer(i, "name")->GetString(), ':');
        if (nameS.size() != 3) continue; // 格式错误
        strReplace(&nameS[0], ".", "/"); // 包名转换路径
        temp.append(nameS[0])
            .append("/")
            .append(nameS[1])
            .append("/")
            .append(nameS[2])
            .append("/")
            .append(nameS[1])
            .append("-")
            .append(nameS[2])
            .append(".jar");
        result.push_back(temp);
    }
    return result;
}

Account addOfflineAccount() {
    Account account;
    return account;
}

Account addOnlineAccount() {
    Account account;
    return account;
}

bool delGame() {
    return true;
}

string getCMD(Account account, Game game, string javaDir, string gameDir) {
    #ifdef DEBUG
        cout << "[Info] mc_core::getCMD : Start." << endl;
    #endif
    // 使用自定义参数：
    if (!game.args.empty()) return javaDir + " " + game.args;    

    #ifdef DEBUG
        cout << "[Info] mc_core::getCMD : OS Type:" << os << ", " << arch << endl;
    #endif

    // 生成参数：

    string result = javaDir + " "; // 结果
    string jvmArg = string("-XX:+UseG1GC ") +
                    "-XX:-UseAdaptiveSizePolicy " +
                    "-XX:-OmitStackTraceInFastThrow " +
                    "-Dfml.ignoreInvalidMinecraftCertificates=True " +
                    "-Dfml.ignorePatchDiscrepancies=True " + 
                    "-Dlog4j2.formatMsgNoLookups=true "; // JVM参数
    string gameArg = ""; // 游戏参数
    vector<string> cp; // -cp内容
    string mainClass = ""; // mainClass值
    string assetIndex = ""; // assetIndex值

    // TODO 添加操作系统信息

    Document doc;
    doc.Parse(openFile(gameDir + "/versions/" + game.version + "/" + game.version + ".json"));
    if (doc.HasParseError()) {
        cout << "[Error] mc_core::getCMD : Failed to get launch command: "
             << gameDir + "/versions/" + game.version + "/" + game.version + ".json"
             << ": in line" << doc.GetErrorOffset() << ": Parse Error." << endl;
        return "";
    }

    // 判断inheritsFrom
    if (doc.HasMember("inheritsFrom")) {
        string parentVer = doc.AtPointer("inheritsFrom")->GetString();
        if (exists(gameDir + "/versions/" + parentVer)) {
            Document par;
            par.Parse(openFile(gameDir + "/versions/" + parentVer + "/" + parentVer + ".json"));
            if (par.HasParseError()) {
                cout << "[Error] mc_core::getCMD : Failed to get launch command: "
                     << gameDir + "/versions/" + parentVer + "/" + parentVer + ".json"
                     << ": in line" << par.GetErrorOffset() << ": Parse Error." << endl;
                return "";
            }
            mainClass = doc.AtPointer("mainClass")->GetString() + " ";
            assetIndex = par.AtPointer("assetIndex", "id")->GetString() + " ";
            cp = addLibs(*par.AtPointer("libraries"), gameDir);
            cp.push_back(gameDir + "/versions/" + parentVer + "/" + parentVer + ".jar:");
            if (!(setArgs(par, &jvmArg, &gameArg) && setArgs(doc, &jvmArg, &gameArg))) return "";
        } else {
            // TODO 下载原版
            return "";
        }
    } else {
        if (!setArgs(doc, &jvmArg, &gameArg)) return "";
        assetIndex = doc.AtPointer("assetIndex", "id")->GetString();
        mainClass = doc.AtPointer("mainClass")->GetString() + " ";
    }

    result.append(jvmArg) // JVM
          .append("${authlib_injector_param} -Xms" + game.xms + " -Xmx" + game.xmx + " ") // 额外JVM
          .append(mainClass) // 主类
          .append(gameArg) // 游戏参数
          .append("--width " + to_string(game.width) + " --height " + to_string(game.height) + " "); // 额外游戏参数

    // Optifine : --tweakClass调整至末尾
    if (result.find("--tweakClass optifine.OptiFineTweaker") != result.npos) {
        strReplace(&result, "--tweakClass optifine.OptiFineTweaker", "");
    }
    if (result.find("--tweakClass optifine.OptiFineForgeTweaker") != result.npos) {
        strReplace(&result, "--tweakClass optifine.OptiFineForgeTweaker", "");
    }

    // 替换result中的模板

    // ${classpath}
    string cps = "";
    vector<string> cp2 = addLibs(*doc.AtPointer("libraries"), gameDir);
    cp.insert(cp.end(), cp2.begin(), cp2.end());
    cp.push_back(gameDir + "/versions/" + game.version + "/" + game.version + ".jar");
    // 去重
    set<string> s(cp.begin(), cp.end());
    cp.assign(s.begin(), s.end());
    for (string s : cp) {
        cps.append(s).append(":");
    }

    strReplace(&result, "${auth_player_name}", account.userName);
    strReplace(&result, "${version_name}", game.version);
    strReplace(&result, "${game_directory}", gameDir);
    strReplace(&result, "${assets_root}", gameDir + "/assets");
    strReplace(&result, "${assets_index_name}", assetIndex);
    strReplace(&result, "${auth_uuid}", account.uuid);
    strReplace(&result, "${auth_access_token}", account.token);
    strReplace(&result, "${user_type}", account.type);
    strReplace(&result, "${version_type}", game.type);
    strReplace(&result, "${natives_directory}", gameDir + "/versions/" + game.version + "/natives");
    strReplace(&result, "${launcher_name}", "\"CE Minecraft Launcher\"");
    strReplace(&result, "${launcher_version}", "1.0.0");
    strReplace(&result, "${classpath}", cps);
    strReplace(&result, "${library_directory}", gameDir + "/libraries");
    strReplace(&result, "${classpath_separator}", ":");
    strReplace(&result, "${authlib_injector_param}", ""); // not support yet

    // TODO natives

    cout << result << endl;

    #ifdef DEBUG
        cout << "[Info] mc_core::getCMD : Finished. Command: " << result << endl;
    #endif
    return result;
}

vector<Game> loadGameList(
    bool reload,
    string gameDir,
    int defHeight,
    int defWidth,
    string defXms,
    string defXmx) {
    vector<Game> gameList;
    if (reload) {
        if (!exists(gameDir + "/versions")) return {};
        vector<string> versionList = getDirs(gameDir + "/versions");
        int count = versionList.size();
        gameList.resize(count);
        for (int i = 0; i < count; i++) {
            // Get version type
            Document doc;
            doc.Parse(openFile(gameDir + "/versions/" + versionList[i] + "/" + versionList[i] + ".json"));
            if (doc.HasParseError()) {
                cout << "[Error] mc_core::loadGameList : Failed to load " << gameDir + "/versions/" + versionList[i] + "/" + versionList[i] + ".json"
                     << ": in line" << doc.GetErrorOffset() << ": has parse error." << endl;
                continue;
            }
            if (!doc.HasMember("type")) {
                cout << "[Error] mc_core::loadGameList : Failed to load " << gameDir + "/versions/" + versionList[i] + "/" + versionList[i] + ".json"
                     << ": doesn't have member \"type\"" << endl;
                continue;
            }

            gameList[i].args = "";
            gameList[i].height = defHeight;
            gameList[i].type = doc.AtPointer("type")->GetString();
            gameList[i].version = versionList[i];
            gameList[i].width = defWidth;
            gameList[i].xms = defXms;
            gameList[i].xmx = defXmx;
        }
        // TODO SHA-1 sum
    } else {
        string cfgText = openFile("index.json");
        if (cfgText.empty()) {
            return loadGameList(true, gameDir, defHeight, defWidth, defXms, defXmx);
        }
        Document doc;
        doc.Parse(cfgText);
        // TODO: if SHA-1 different, reload
        if (doc.HasParseError()) {
            return loadGameList(true, gameDir, defHeight, defWidth, defXms, defXmx);
        }

        Node * list = doc.AtPointer("gameList");
        for (int i = 0; list->AtPointer(i) != nullptr; i++) {
            Game g;
            g.args = list->AtPointer(i, "args")->GetString();
            g.height = list->AtPointer(i, "height")->GetInt64();
            g.type = list->AtPointer(i, "type")->GetString();
            g.version = list->AtPointer(i, "version")->GetString();
            g.width = list->AtPointer(i, "width")->GetInt64();
            g.xms = list->AtPointer(i, "xms")->GetString();
            g.xmx = list->AtPointer(i, "xmx")->GetString();
            gameList.push_back(g);
        }
    }
    return gameList;
}

bool setArgs(Node &n, string *jvmArg, string *gameArg) {
    if (!n.HasMember("arguments") && n.HasMember("minecraftArguments")) {
        // ver < 1.13
        jvmArg->append("-Djava.library.path=${natives_directory} -cp ${classpath} ");
        gameArg->append(n.AtPointer("minecraftArguments")->GetString());
    } else if (n.HasMember("arguments") && n.AtPointer("arguments")->HasMember("jvm") && n.AtPointer("arguments")->HasMember("game")) {
        // ver >= 1.13
        jvmArg->append(addArgs(*n.AtPointer("arguments", "jvm")));
        gameArg->append(addArgs(*n.AtPointer("arguments", "game")));
    } else {
        // 错误
        cout << "[Error] mc_core::setArgs : Failed to get jvm and game arguments: Format Error." << endl;
        return false;
    }
    return true;
}
