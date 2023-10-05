#[derive(Debug, PartialEq)]
pub enum OpCode {
    CLR,              //clear screen
    JMP(u16),         //1NNN jmp to NNN
    SET(u8, u8),      //6XNN set register VX , X is addr in v_registers of 0-F
    ADD(u8, u8),      //7XNN add value to register VX
    SetAddrReg(u16),  //ANNN set index register I
    DXYN(u8, u8, u8), //display/draw
    UNFINISHED,
}
