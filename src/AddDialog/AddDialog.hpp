#ifndef AddDialog_H
#define AddDialog_H

#include <iostream>
#include <QDialog>
#include <QObject>

QT_BEGIN_NAMESPACE
namespace Ui { class AddDialog; }
QT_END_NAMESPACE

using namespace std;

class AddDialog : public QDialog
{
    Q_OBJECT

public slots:

public:
    AddDialog(QWidget *parent = nullptr);
    ~AddDialog();

private:
    Ui::AddDialog *ui;
    
};
#endif // AddDialog_H
