#ifndef CEMCL_H
#define CEMCL_H

#include <iostream>
#include <QMainWindow>
#include <QObject>
#include <vector>

QT_BEGIN_NAMESPACE
namespace Ui { class CEMCL; }
QT_END_NAMESPACE

using namespace std;

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
    Ui::CEMCL *ui;
    struct Config
    {
        bool isOnline = false;
        string account = "Steve";
        string gameDir = ".minecraft";
        string javaDir = "";
        string token = "";
    };
    Config cfg;
    vector<vector<string> > versionList;
    bool loadConfig();
    bool loadVersionList(bool ignoreIndexFile);
    
};
#endif // CEMCL_H
