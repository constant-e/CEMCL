#include "mc_core.hpp"

#include "file/file.hpp"
#include "network/network.hpp"
#include "strTools/strTools.hpp"

using sonic_json::Document;
using std::cout;
using std::endl;
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
    cout << "[Warning] mc_core: Your OS or compiler may not be supported." << endl;
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
    cout << "[Warning] mc_core: Your OS arch or compiler may not be supported." << endl;
#endif

string addArgs(Node & n) {
    string result = "";
    for (int i = 0; n.AtPointer(i) != nullptr; i++) {
        if (n.AtPointer(i)->IsString()) {
            // 无限制，直接添加
            result.append(n.AtPointer(i)->GetString()).append(" ");
            continue;
        }

        // 不是String，判断rules
        if (!n.AtPointer(i)->HasMember("rules")) continue; // 格式错误
        Node * r = n.AtPointer(i, "rules");
        for (int j = 0; r->AtPointer(j) != nullptr; j++) {
            if (!r->AtPointer(j)->HasMember("action")) continue;
            if (r->AtPointer(j)->HasMember("features")) continue; // not support yet
            if (r->AtPointer(j, "action")->GetString() == "allow") {
                if (r->AtPointer(j, "os")->HasMember("name") &&
                    r->AtPointer(j, "os", "name")->GetString() != os) {
                    continue;
                }
                if (r->AtPointer(j, "os")->HasMember("arch") &&
                    r->AtPointer(j, "os", "arch")->GetString() != arch) {
                    continue;
                }
            } else if (r->AtPointer(j, "action")->GetString() == "disallow") {
                if (r->AtPointer(j, "os")->HasMember("name") &&
                    r->AtPointer(j, "os", "name")->GetString() == os) {
                    continue;
                }
                if (r->AtPointer(j, "os")->HasMember("arch") &&
                    r->AtPointer(j, "os", "arch")->GetString() == arch) {
                    continue;
                }
            }

            if (!n.AtPointer(i)->HasMember("value")) continue; // 格式错误
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
    }
    return result;
}

bool addGame() {
    return true;
}

// TODO 下载相关
string addLibs(Node & n, string gameDir) {
    string result = "";
    for (int i = 0; n.AtPointer(i) != nullptr; i++) {
        string temp = gameDir + "/libraries/";
        bool allow = true;
        if (n.AtPointer(i)->HasMember("rules")) {
            // 有rules，进行判断
            Node * r = n.AtPointer(i, "rules");
            for (int j = 0; r->AtPointer(j) != nullptr; j++) {
                if (!r->AtPointer(j)->HasMember("action")) allow = false;
                if (r->AtPointer(j, "action")->GetString() == "allow") {
                    if (r->AtPointer(j, "os")->HasMember("name") &&
                        r->AtPointer(j, "os", "name")->GetString() != os) {
                        allow = false;
                    }
                    if (r->AtPointer(j, "os")->HasMember("arch") &&
                        r->AtPointer(j, "os", "arch")->GetString() != arch) {
                        allow = false;
                    }
                } else if (r->AtPointer(j, "action")->GetString() == "disallow") {
                    if (r->AtPointer(j, "os")->HasMember("name") &&
                        r->AtPointer(j, "os", "name")->GetString() == os) {
                        allow = false;
                    }
                    if (r->AtPointer(j, "os")->HasMember("arch") &&
                        r->AtPointer(j, "os", "arch")->GetString() == arch) {
                        allow = false;
                    }
                }
            }
        }
        if (!allow) continue;
        if (!n.AtPointer(i)->HasMember("name")) continue; // 格式错误

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
              .append(".jar;");
        result.append(temp);
    }
    result.append("\b "); // 去掉末尾的;
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
    string result = javaDir +
    " -XX:+UseG1GC " +
    "-XX:-UseAdaptiveSizePolicy " +
    "-XX:-OmitStackTraceInFastThrow " +
    "-Dfml.ignoreInvalidMinecraftCertificates=True " +
    "-Dfml.ignorePatchDiscrepancies=True " + 
    "-Dlog4j2.formatMsgNoLookups=true ";

    // TODO 添加操作系统信息

    string path = gameDir + "/versions/" + game.version + "/" + game.version + ".json";
    Document doc;
    doc.Parse(openFile(path));
    if (doc.HasParseError()) {
        cout << "[Error] mc_core::getCMD : Failed to get launch command: " << path
        << ": in line" << doc.GetErrorOffset() << ": Parse Error." << endl;
        return "";
    }

    Node * mcArg; // 游戏参数Node
    // JVM参数
    if (!doc.HasMember("arguments")) {
        // 1.12.2-
        result.append("-Djava.library.path=${natives_directory} -cp ${classpath} ");
        if (!doc.HasMember("minecraftArguments")) {
            cout << "[Error] mc_core::getCMD : Failed to get launch command: " << path
            << ": member \"minecraftArguments\" is not found." << endl;
            return "";
        }
        mcArg = doc.AtPointer("minecraftArguments");
    } else if (doc.HasMember("arguments")) {
        // 1.13.2+
        if (!doc.AtPointer("arguments")->HasMember("jvm")) {
            cout << "[Error] mc_core::getCMD : Failed to get launch command: " << path
            << ": member \"jvm\" is not found." << endl;
            return "";
        }
        Node * jvm = doc.AtPointer("arguments", "jvm");
        result.append(addArgs(*jvm));
        if (!doc.AtPointer("arguments")->HasMember("game")) {
            cout << "[Error] mc_core : Failed to get launch command: " << path
            << ": not has mc arguments" << endl;
            return "";
        }
        mcArg = doc.AtPointer("arguments", "game");
    } else {
        // 错误
        cout << "[Error] mc_core : Failed to get launch command: " << path
        << ": format error." << endl;
        return "";
    }
    result.append("${authlib_injector_param} ");

    // 游戏主类
    result.append("-Xms" + game.xms + " -Xmx" + game.xmx + " " + doc.AtPointer("mainClass")->GetString() + " ");

    // 游戏参数
    result.append(addArgs(*mcArg));

    // Optifine --tweakClass调整至末尾
    if (result.find("--tweakClass optifine.OptiFineTweaker") != result.npos) {
      strReplace(&result, "--tweakClass optifine.OptiFineTweaker", "");
    }
    if (result.find("--tweakClass optifine.OptiFineForgeTweaker") != result.npos) {
        strReplace(&result, "--tweakClass optifine.OptiFineForgeTweaker", "");
    }

    // width 和 height
    result.append("--width " + to_string(game.width) + " --height " + to_string(game.height) + " ");

    // 替换result中的模板

    // ${classpath}
    string cp = addLibs(*doc.AtPointer("libraries"), gameDir);

    // ${assets_index_name}
    string assetIndex = doc.AtPointer("assetIndex", "id")->GetString();

    strReplace(&result, "${auth_player_name}", account.userName);
    strReplace(&result, "${version_name}", game.version);
    strReplace(&result, "${game_directory}", gameDir.append("/versions/").append(game.version));
    strReplace(&result, "${assets_root}", gameDir.append("/assets"));
    strReplace(&result, "${assets_index_name}", assetIndex);
    strReplace(&result, "${auth_uuid}", account.uuid);
    strReplace(&result, "${auth_access_token}", account.token);
    strReplace(&result, "${user_type}", account.online ? "Mojang" : "Legacy");
    strReplace(&result, "${version_type}", game.type);
    strReplace(&result, "${natives_directory}", "");
    strReplace(&result, "${launcher_name}", "\"CE Minecraft Launcher\"");
    strReplace(&result, "${launcher_version}", "1.0.0");
    strReplace(&result, "${classpath}", cp);
    strReplace(&result, "${library_directory}", gameDir + "/libraries");
    strReplace(&result, "${classpath_separator}", ";");
    strReplace(&result, "${authlib_injector_param}", ""); // offline

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
