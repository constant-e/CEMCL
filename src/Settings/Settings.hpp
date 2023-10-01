#ifndef Settings_H
#define Settings_H

#include <iostream>
#include <QDialog>
#include <QObject>

QT_BEGIN_NAMESPACE
namespace Ui { class Settings; }
QT_END_NAMESPACE

using namespace std;

class Settings : public QDialog
{
    Q_OBJECT

public slots:

public:
    Settings(QWidget *parent = nullptr);
    ~Settings();

private:
    Ui::Settings *ui;
    
};
#endif // Settings_H
