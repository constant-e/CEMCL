#include "mc_core.hpp"

#include "file/file.hpp"
#include "network/network.hpp"
#include "sonic/sonic.h"
#include "strTools/strTools.hpp"

using sonic_json::Document;
using sonic_json::Node;
using std::cout;
using std::endl;

// 已有变量：gameDir - .minecraft位置    javaDir - java位置(可执行文件)

bool addGame() {
    return true;
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
    
    string os = "";
    // 先确定系统
    #ifdef _WIN32
        os = "windows";
    #elif __APPLE__
        os = "osx";
    #elif __linux__
        os = "linux";
    #else
        cout << "[Warning] mc_core: Your OS may not be supported." << endl;
    #endif

    #ifdef DEBUG
        cout << "[Info] mc_core::getCMD : OS Type:" << os << endl;
    #endif

    // 生成参数：
    string result = javaDir +
    " -XX:+UseG1GC " +
    "-XX:-UseAdaptiveSizePolicy " +
    "-XX:-OmitStackTraceInFastThrow " +
    "-Dfml.ignoreInvalidMinecraftCertificates=True " +
    "-Dfml.ignorePatchDiscrepancies=True " + 
    "-Dlog4j2.formatMsgNoLookups=true ";

    // Windows需要：
    #ifdef _WIN32
        result.append("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump ")
    #endif

    // TODO 添加操作系统信息

    string path = gameDir + "/versions/" + game.version + "/" + game.version + ".json";
    Document doc;
    doc.Parse(openFile(path));
    if (doc.HasParseError()) {
        cout << "[Error] mc_core : Failed to get launch command: " << path
        << ": in line" << doc.GetErrorOffset() << ": has parse error." << endl;
        return "";
    }

    Node * mcArg; // 游戏参数Node
    // JVM参数
    if (!doc.HasMember("arguments")) {
        // 1.12.2-
        result.append("-Djava.library.path=${natives_directory} -cp ${classpath} ");
        if (!doc.HasMember("minecraftArguments")) {
            cout << "[Error] mc_core : Failed to get launch command: " << path
            << ": not has mc arguments" << endl;
            return "";
        }
        mcArg = doc.AtPointer("minecraftArguments");
    } else if (doc.HasMember("arguments") && doc.AtPointer("arguments")->HasMember("jvm")) {
        // 1.13.2+
        Node * jvm = doc.AtPointer("arguments", "jvm");
        for (int i = 0; jvm->AtPointer(i) != nullptr; i++) {
            if (jvm->AtPointer(i)->IsString()) {
                // 无rule
                result.append(jvm->AtPointer(i)->GetString());
                result.append(" ");
            } else {
                // 有rule
            }
        }
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
    for (int i = 0; mcArg->AtPointer(i) != nullptr; i++) {
        if (mcArg->AtPointer(i)->IsString()) {
        // 无rule
            result.append(mcArg->AtPointer(i)->GetString());
            result.append(" ");
        } else {
            // 有rule
        }
    }

    // Optifine --tweakClass调整至末尾
    if (result.find("--tweakClass optifine.OptiFineTweaker") != result.npos) {
      strReplace(&result, "--tweakClass optifine.OptiFineTweaker", "");
    }
    if (result.find("--tweakClass optifine.OptiFineForgeTweaker") != result.npos) {
        strReplace(&result, "--tweakClass optifine.OptiFineForgeTweaker", "");
    }

    // width 和 height
    result.append("--width " + game.width).append(" --height " + game.height).append(" ");

    // 替换result中的模板

    // ${classpath}
    string cp = "";
    for (int i = 0; doc.AtPointer("libraries", i) != nullptr; i++) {
        Node * n = doc.AtPointer("libraries", i);
        if (n->HasMember("rules")) {
            for (int j = 0; n->AtPointer("rules", j) != nullptr; j++) {
                if (n->AtPointer("rules", j, "action")->GetString() == "allow") {
                    if (n->AtPointer("rules", j, "os", "name")->GetString() != os) {
                        continue;
                    }
                } else if (n->AtPointer("rules", j, "action")->GetString() == "disallow") {
                    if (n->AtPointer("rules", j, "os", "name")->GetString() == os) {
                        continue;
                    }
                }
            }
        }
        if (!n->HasMember("name") ||
        n->HasMember("natives") ||
        !n->HasMember("downloads") ||
        (n->AtPointer("downloads")->HasMember("classifiers") && !n->AtPointer("downloads")->HasMember("artifact"))) {
            continue;
        }
        string p = n->AtPointer("downloads", "artifact", "path")->GetString();
        vector<string> pS = splitStr(p, '/');
        // TODO 完成文件n验证
        // if (isSame(gameDir + "/libraries/" + p, pS[pS.size()], n->AtPointer("downloads", "artifact", "sha1")->GetString())) {
        //     cout << "sha1 same." << endl;*/
        //     cp.append(gameDir + "/libraries/" + p + ";");
        // } else {
        //     cout << 1 << endl;
        // }
        cp.append(gameDir + "/libraries/" + p + ";");
    }
    cp.append("\b ");

    // TODO 完成剩余参数替换

    strReplace(&result, "${auth_player_name}", account.userName);
    strReplace(&result, "${version_name}", game.version);
    strReplace(&result, "${game_directory}", gameDir.append("/versions/").append(game.version));
    strReplace(&result, "${assets_root}", gameDir.append("/assets"));
    strReplace(&result, "${assets_index_name}", game.version);
    strReplace(&result, "${auth_uuid}", account.uuid);
    strReplace(&result, "${auth_access_token}", account.token);
    strReplace(&result, "${user_type}", account.online ? "Mojang" : "Legacy");
    strReplace(&result, "${version_type}", game.type);
    strReplace(&result, "${natives_directory}", "");
    strReplace(&result, "${launcher_name}", "CE Minecraft Launcher");
    strReplace(&result, "${launcher_version}", "1.0.0");
    strReplace(&result, "${classpath}", cp);
    strReplace(&result, "${library_directory}", gameDir + "/libraries");
    strReplace(&result, "${classpath_separator}", ";");
    strReplace(&result, "${authlib_injector_param}", ""); // offline

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
