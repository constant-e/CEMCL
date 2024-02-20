#pragma once

#include <iostream>
#include <QDialog>
#include <QObject>

QT_BEGIN_NAMESPACE
namespace Ui { class AddVerDialog; }
QT_END_NAMESPACE

class AddVerDialog : public QDialog
{
    Q_OBJECT

public slots:

public:
    AddVerDialog(QWidget *parent = nullptr);
    ~AddVerDialog();

private:
    Ui::AddVerDialog *UI;
    
};
