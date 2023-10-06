#ifndef ConfigureDialog_H
#define ConfigureDialog_H

#include <iostream>
#include <QDialog>
#include <QObject>

QT_BEGIN_NAMESPACE
namespace Ui { class ConfigureDialog; }
QT_END_NAMESPACE

using namespace std;

class ConfigureDialog : public QDialog
{
    Q_OBJECT

public slots:

public:
    ConfigureDialog(QWidget *parent = nullptr);
    ~ConfigureDialog();

private:
    Ui::ConfigureDialog *ui;
    
};
#endif // ConfigureDialog_H
