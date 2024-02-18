#ifndef CEMCL_H
#define CEMCL_H

#include <iostream>
#include <QMainWindow>
#include <QObject>
#include <vector>

#include "mc_core/mc_core.hpp"

#define DEFAULTCFG "{\"closeAfterLaunch\":false,\"forgeSource\":\"\",\"gameDir\":\".minecraft\",\"javaDir\":\"java\",\"MCSource\":\"\",\"optifineSource\":\"\",\"xms\":\"1G\",\"xmx\":\"2G\",\"width\":800,\"height\":600}"

QT_BEGIN_NAMESPACE
namespace Ui { class CEMCL; }
QT_END_NAMESPACE

class CEMCL : public QMainWindow
{
    Q_OBJECT

public slots:
    void onClickAddBtn();
    void onClickConfigureBtn();
    void onClickSettingsBtn();
    void onClickStartBtn();

public:
    CEMCL(QWidget *parent = nullptr);
    ~CEMCL();

private:
    // account
    vector<Account> accountList;

    // config
    bool closeAfterLaunch;

    // string fabricSource; not support yet

    string forgeSource;
    string gameDir;
    int height;
    // path to java or javaw, not dir
    string javaDir;

    // string liteLoaderSource; not support yet

    string MCSource;
    string optfineSource;

    // string quiltSource; not support yet
    
    int width;
    string xms;
    string xmx;
    
    // UI
    Ui::CEMCL *ui;

    //game
    vector<Game> gameList;

    // functions
    bool loadAccount();
    bool loadConfig();
    bool loadUI();
};
#endif // CEMCL_H
