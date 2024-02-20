#pragma once

#include <iostream>
#include <QDialog>
#include <QObject>

QT_BEGIN_NAMESPACE
namespace Ui { class AddAccDialog; }
QT_END_NAMESPACE

class AddAccDialog : public QDialog
{
    Q_OBJECT

public slots:

public:
    AddAccDialog(QWidget *parent = nullptr);
    ~AddAccDialog();

private:
    Ui::AddAccDialog *UI;
    
};
