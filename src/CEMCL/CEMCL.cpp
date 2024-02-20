#include <QMessageBox>
#include <QObject>
#include <QTableWidget>

#include "AddAccDialog/AddAccDialog.hpp"
#include "AddVerDialog/AddVerDialog.hpp"
#include "CEMCL.hpp"
#include "CEMCLUI.hpp"
#include "EditAccDialog/EditAccDialog.hpp"
#include "EditVerDialog/EditVerDialog.hpp"
#include "file/file.hpp"
#include "Settings/Settings.hpp"

using sonic_json::Document;
using std::cout;
using std::endl;

bool CEMCL::loadAccount() {
    #ifdef DEBUG
        cout << "[Info] CEMCL::loadAccount : Loading account.json ..." << endl;
    #endif
    string accText = openFile("account.json");
    #ifdef DEBUG
        cout << "[Info] CEMCL::loadAccount : Successfully read account.json." << endl;
    #endif
    if (accText.empty()) {
        #ifdef DEBUG
            cout << "[Info] CEMCL::loadAccount : account.json is empty. Using DEFAULTACC." << endl;
        #endif
        saveFile("account.json", DEFAULTACC);
        return loadAccount();
    } else {
        Document doc;
        doc.Parse(accText);
        if (doc.HasParseError()) {
            cout << "[Warning] CEMCL::loadAccount : Failed to load account.json : "
                 << "parse error in line " << doc.GetParseError() << ". Using default config." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                QString("Failed to load account.json : parse error in line " + doc.GetParseError()).append(". Using default config.")); 
            return true;
        }
        #ifdef DEBUG
            cout << "[Info] CEMCL::loadAccount : Successfully parsed account.json." << endl;
        #endif

        for (int i = 0; doc.AtPointer(i) != nullptr; i++) {
            Account account;
            if (doc.AtPointer(i)->HasMember("type")) {
                account.type = doc.AtPointer(i, "type")->GetString();
            } else {
                cout << "[Error] CEMCL::loadAccount : account doesn't have member "
                     << "\"type\". Skipping." << endl;
                continue;
            }
            
            if (doc.AtPointer(i)->HasMember("token")) {
                account.token = doc.AtPointer(i, "token")->GetString();
            } else {
                cout << "[Error] CEMCL::loadAccount : account doesn't have member "
                     << "\"token\". Skipping." << endl;
                continue;
            }

            if (doc.AtPointer(i)->HasMember("userName")) {
                account.userName = doc.AtPointer(i, "userName")->GetString();
            } else {
                cout << "[Error] CEMCL::loadAccount : account doesn't have member "
                     << "\"userName\". Skipping." << endl;
                continue;
            }

            if (doc.AtPointer(i)->HasMember("uuid")) {
                account.uuid = doc.AtPointer(i, "uuid")->GetString();
            } else {
                cout << "[Error] CEMCL::loadAccount : account doesn't have member "
                     << "\"uuid\". Skipping." << endl;
                continue;
            }
            accountList.push_back(account);
        }
    }

    if (accountList.empty()) {
        cout << "[Error] CEMCL::loadAccount : Failed to load account.json : No vailed account available." << endl;
        QMessageBox::warning(
            this, 
            "Error", 
            QString("Failed to load account.json : No vailed account available."));
        return false;
    } 

    #ifdef DEBUG
        cout << "[Info] CEMCL::loadAccount : Successfully loaded account. Current accounts are:\r"
        for (Account a : accountList) {
            cout << "online: " << a.online << "\r"
                 << "token: " << a.token << "\r"
                 << "userName: " << a.userName << "\r"
                 << "---End---" << endl;
        }
    #endif
    return true;
}

bool CEMCL::loadConfig() {
    #ifdef DEBUG
        cout << "[Info] CEMCL::loadConfig : Loading config.json ..." << endl;
    #endif
    // load config.json
    string cfgText = openFile("config.json");
    #ifdef DEBUG
        cout << "[Info] CEMCL::loadConfig : Successfully read config.json." << endl;
    #endif
    if (cfgText.empty()) {
        #ifdef DEBUG
            cout << "[Info] CEMCL::loadConfig : config.json is empty. Using DEFAULTCFG." << endl;
        #endif
        saveFile("config.json", DEFAULTCFG);
    } else {
        Document doc;
        doc.Parse(cfgText);
        if (doc.HasParseError()) {
            cout << "[Warning] CEMCL::loadConfig : Failed to load config.json : "
                 << "parse error in line " << doc.GetParseError() << ". Using default config." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                QString("Failed to load config.json : parse error in line " + doc.GetParseError()).append(". Using default config.")); 
            return true;
        }
        #ifdef DEBUG
            cout << "[Info] CEMCL::loadConfig : Successfully parsed config.json." << endl;
        #endif

        if (doc.HasMember("closeAfterLaunch")) {
            closeAfterLaunch = doc.AtPointer("closeAfterLaunch")->GetBool();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"closeAfterLaunch\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"closeAfterLaunch\". Using the default value.");
        }

        if (doc.HasMember("forgeSource")) {
            forgeSource = doc.AtPointer("forgeSource")->GetString();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"forgeSource\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"forgeSource\". Using the default value.");
        }
        
        if (doc.HasMember("gameDir")) {
            gameDir = doc.AtPointer("gameDir")->GetString();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"gameDir\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"gameDir\". Using the default value.");
        }

        if (doc.HasMember("height")) {
            height = doc.AtPointer("height")->GetInt64();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"height\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"height\". Using the default value.");
        }

        if (doc.HasMember("javaDir")) {
            javaDir = doc.AtPointer("javaDir")->GetString();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"javaDir\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"javaDir\". Using the default value.");
        }

        if (doc.HasMember("MCSource")) {
            MCSource = doc.AtPointer("MCSource")->GetString();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"MCSource\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"MCSource\". Using the default value.");
        }

        if (doc.HasMember("width")) {
            width = doc.AtPointer("width")->GetInt64();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"width\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"width\". Using the default value.");
        }

        if (doc.HasMember("xms")) {
            xms = doc.AtPointer("xms")->GetString();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"xms\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"xms\". Using the default value.");
        }

        if (doc.HasMember("xmx")) {
            xmx = doc.AtPointer("xmx")->GetString();
        } else {
            cout << "[Warning] CEMCL::loadConfig : config.json doesn't have member "
                 << "\"xmx\". Using the default value." << endl;
            QMessageBox::warning(
                this, 
                "Warning", 
                "config.json doesn't have member \"xmx\". Using the default value.");
        }
    }
    #ifdef DEBUG
        cout << "[Info] CEMCL::loadConfig : Successfully loaded config. Current configs are:\r"
             << "closeAfterLaunch: " << closeAfterLaunch << "\r"
             << "forgeSource: " << forgeSource << "\r"
             << "gameDir: " << gameDir << "\r"
             << "height: " << height << "\r"
             << "javaDir: " << javaDir << "\r"
             << "MCSource: " << MCSource << "\r"
             << "width: " << width << "\r"
             << "xms: " << xms << "\r"
             << "xmx: " << xmx << endl;
    #endif
    return true;
}

bool CEMCL::loadUI() {
    #ifdef DEBUG
        cout << "[Info] CEMCL::loadUI : Loading UI ..." << endl;
    #endif
    // accTab
    int accCount = accountList.size();
    UI->AccTableWidget->setRowCount(accCount);
    UI->AccTableWidget->setHorizontalHeaderItem(0, new QTableWidgetItem("Name"));
    UI->AccTableWidget->setHorizontalHeaderItem(1, new QTableWidgetItem("Type"));
    for (int i = 0; i < accCount; i++) {
        QTableWidgetItem * item1 = new QTableWidgetItem();
        item1->setText(QString(accountList[i].userName.c_str()));
        UI->AccTableWidget->setItem(i, 0, item1);

        QTableWidgetItem * item2 = new QTableWidgetItem();
        item2->setText(QString(accountList[i].type.c_str()));
        UI->AccTableWidget->setItem(i, 1, item2);
    }

    // verTab
    int c = gameList.size();
    UI->VerTableWidget->setRowCount(c);
    UI->VerTableWidget->setHorizontalHeaderItem(0, new QTableWidgetItem("Version"));
    UI->VerTableWidget->setHorizontalHeaderItem(1, new QTableWidgetItem("Type"));
    UI->VerTableWidget->setHorizontalHeaderItem(2, new QTableWidgetItem("Describe"));
    for (int i = 0; i < c; i++) {
        QTableWidgetItem * item1 = new QTableWidgetItem();
        item1->setText(QString(gameList[i].version.c_str()));
        UI->VerTableWidget->setItem(i, 0, item1);

        QTableWidgetItem * item2 = new QTableWidgetItem();
        item2->setText(QString(gameList[i].type.c_str()));
        UI->VerTableWidget->setItem(i, 1, item2);
    }
    #ifdef DEBUG
        cout << "[Info] CEMCL::loadUI : Successfully loaded UI." << endl;
    #endif
    return true;
}

void CEMCL::onClickEditBtn() {
    #ifdef DEBUG
        cout << "[Info] CEMCL::onClickEditBtn : Triggered." << endl;
    #endif
    if (UI->AccTableWidget->currentRow() != -1 &&
        UI->VerTableWidget->currentRow() == -1) {
        // edit account
        EditAccDialog * d = new EditAccDialog(this);
        d->show();
        d->exec();
        UI->AccTableWidget->setCurrentCell(-1, -1);
    } else if (UI->AccTableWidget->currentRow() == -1 &&
               UI->VerTableWidget->currentRow() != -1) {
        // edit version
        EditVerDialog * d = new EditVerDialog(this);
        d->show();
        d->exec();
        UI->VerTableWidget->setCurrentCell(-1, -1);
    } else {
        // error
        #ifdef DEBUG
            cout << "[Error] CEMCL::onClickEditBtn : Targets too few or too many." << endl;
        #endif
        QMessageBox::warning(
            this, 
            "Error", 
            "Please select one account or Minecraft version.");
        UI->AccTableWidget->setCurrentCell(-1, -1);
        UI->VerTableWidget->setCurrentCell(-1, -1);
        return;
    }
}

void CEMCL::onClickNewBtn() {
    #ifdef DEBUG
        cout << "[Info] CEMCL::onClickAddBtn : Triggered." << endl;
    #endif
    if (UI->AccTableWidget->currentRow() != -1 &&
        UI->VerTableWidget->currentRow() == -1) {
        // new account
        AddAccDialog * d = new AddAccDialog(this);
        d->show();
        d->exec();
        UI->AccTableWidget->setCurrentCell(-1, -1);
    } else if (UI->AccTableWidget->currentRow() == -1 &&
               UI->VerTableWidget->currentRow() != -1) {
        // edit version
        AddVerDialog * d = new AddVerDialog(this);
        d->show();
        d->exec();
        UI->VerTableWidget->setCurrentCell(-1, -1);
    } else {
        // error
        #ifdef DEBUG
            cout << "[Error] CEMCL::onClickNewBtn : Haven't select target." << endl;
        #endif
        QMessageBox::warning(
            this, 
            "Error", 
            "Please select \"Account\" table or \"Minecraft Version\" table.");
        UI->AccTableWidget->setCurrentCell(-1, -1);
        UI->VerTableWidget->setCurrentCell(-1, -1);
        return;
    }
}

void CEMCL::onClickSettingsBtn() {
    #ifdef DEBUG
        cout << "[Info] CEMCL::onClickSettingsBtn : Triggered." << endl;
    #endif
    Settings * s = new Settings(this);
    s->show();
    s->exec();
}

void CEMCL::onClickStartBtn() {
    #ifdef DEBUG
        cout << "[Info] CEMCL::onClickStartBtn : Triggered." << endl;
    #endif
    // TODO 下载
    int accIndex = UI->AccTableWidget->currentRow();
    int verIndex = UI->VerTableWidget->currentRow();
    if (accIndex == -1) {
        #ifdef DEBUG
            cout << "[Error] CEMCL::onClickStartBtn : Haven't select account yet." << endl;
        #endif
        QMessageBox::warning(
            this, 
            "Error", 
            "Haven't select account yet. Please select one first.");
        return;
    }
    if (verIndex == -1) {
        #ifdef DEBUG
            cout << "[Error] CEMCL::onClickStartBtn : Haven't select game yet." << endl;
        #endif
        QMessageBox::warning(
            this, 
            "Error", 
            "Haven't select game yet. Please select one first.");
        return;
    }
    system(getCMD(accountList[accIndex], gameList[verIndex], javaDir, gameDir).append(" &").c_str());
    UI->AccTableWidget->setCurrentCell(-1, -1);
    UI->VerTableWidget->setCurrentCell(-1, -1);
}

CEMCL::CEMCL(QWidget *parent)
    : QMainWindow(parent)
    , UI(new Ui::CEMCL) {
    #ifdef DEBUG
        cout << "[Info] CEMCL::CEMCL : APP start." << endl;
    #endif
    if (!loadAccount()) return; // 账户
    if (!loadConfig()) return; // 启动器配置文件 + 游戏默认配置
    gameList = loadGameList(false, gameDir, 600, 800, "1G", "2G"); // 游戏列表
    // TODO UI设计确定后，合并以下内容至loadUI()，添加双语言。
    UI->setupUi(this);
    if (!loadUI()) return; // UI
    QObject::connect(UI->EditBtn, &QPushButton::clicked, this, &CEMCL::onClickEditBtn);
    QObject::connect(UI->NewBtn, &QPushButton::clicked, this, &CEMCL::onClickNewBtn);
    QObject::connect(UI->SettingsBtn, &QPushButton::clicked, this, &CEMCL::onClickSettingsBtn);   
    QObject::connect(UI->StartBtn, &QPushButton::clicked, this, &CEMCL::onClickStartBtn);
    #ifdef DEBUG
        cout << "[Info] CEMCL::CEMCL : Finished loading." << endl;
    #endif
}

CEMCL::~CEMCL() {
    #ifdef DEBUG
        cout << "[Info] CEMCL::~CEMCL : Closing APP." << endl;
    #endif
    // TODO 更新配置文件
    delete UI;
}
