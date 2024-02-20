#pragma once

#include <iostream>
#include <QDialog>
#include <QObject>

QT_BEGIN_NAMESPACE
namespace Ui { class EditVerDialog; }
QT_END_NAMESPACE

class EditVerDialog : public QDialog
{
    Q_OBJECT

public slots:

public:
    EditVerDialog(QWidget *parent = nullptr);
    ~EditVerDialog();

private:
    Ui::EditVerDialog *UI;
    
};
